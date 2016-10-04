//! Helper functions and types we use for (de)serialization.  These handle
//! several common, annoying patterns in the `docker-compose.yml` format.

use regex::Regex;
use serde::de;
use serde::de::{Deserialize, Deserializer, MapVisitor, SeqVisitor, Visitor};
use std::collections::BTreeMap;
use std::marker::PhantomData;

use super::interpolation::{InterpolatableValue, RawOr, raw};

/// Test whether a value is false.  Used to determine when to serialize
/// things.
pub fn is_false(b: &bool) -> bool {
    !b
}

/// Normalize YAML-format data for comparison purposes.  Used by unit
/// tests.
#[cfg(test)]
#[cfg_attr(feature="clippy", allow(trivial_regex))]
pub fn normalize_yaml(yaml: &str) -> String {
    lazy_static! {
        // Match a key/value pair.
        static ref WS_NL: Regex =
            Regex::new(" +\n").unwrap();

        static ref NL_EOS: Regex =
            Regex::new("\n$").unwrap();
    }

    NL_EOS.replace_all(&WS_NL.replace_all(yaml, "\n"), "")
}

/// Certain maps in `docker-compose.yml` files may be specified in two
/// forms.  The first form is an ordinary map:
///
/// ```yaml
/// args:
///   foo: 1
///   bar: 2
/// ```
///
/// The second form is a list of key/value pairs:
///
/// ```yaml
/// args:
///   - "foo=1"
///   - "bar=2"
/// ```
///
/// To expand these, you should (theoretically) be able to use:
///
/// ```text
/// struct Example {
///     #[serde(deserialize_with = "deserialize_hash_or_key_value_list")]
///     pub args: BTreeMap<String, String>,
/// }
/// ```
pub fn deserialize_map_or_key_value_list<D>
    (deserializer: &mut D)
     -> Result<BTreeMap<String, String>, D::Error>
    where D: Deserializer
{
    /// Declare an internal visitor type to handle our input.
    struct MapOrKeyValueListVisitor;

    impl Visitor for MapOrKeyValueListVisitor {
        type Value = BTreeMap<String, String>;

        // We have a real map.
        fn visit_map<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: MapVisitor
        {
            let mut map: BTreeMap<String, String> = BTreeMap::new();
            while let Some(key) = try!(visitor.visit_key::<String>()) {
                if map.contains_key(&key) {
                    let msg = format!("duplicate map key: {}", &key);
                    return Err(<V::Error as de::Error>::custom(msg));
                }
                // Work around https://github.com/serde-rs/serde/issues/528
                //
                // TODO BLOCKED: Apply a better fix for error messages.
                match visitor.visit_value::<String>() {
                    Ok(val) => {
                        map.insert(key, val);
                    }
                    Err(_) => {
                        let msg = "Expected string value in key/value map";
                        return Err(<V::Error as de::Error>::custom(msg.to_owned()));
                    }
                }
            }
            try!(visitor.end());
            Ok(map)
        }

        // We have a key/value list.  Yuck.
        fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: SeqVisitor
        {
            lazy_static! {
                // Match a key/value pair.
                static ref KEY_VALUE: Regex =
                    Regex::new("^([^=]+)=(.*)$").unwrap();
            }

            let mut map: BTreeMap<String, String> = BTreeMap::new();
            while let Some(key_value) = try!(visitor.visit::<String>()) {
                let caps = try!(KEY_VALUE.captures(&key_value).ok_or_else(|| {
                    let msg = format!("expected KEY=value, got: <{}>", &key_value);
                    <V::Error as de::Error>::custom(msg)
                }));
                let key = caps.at(1).unwrap();
                let value = caps.at(2).unwrap();
                if map.contains_key(key) {
                    let msg = format!("duplicate map key: {}", key);
                    return Err(<V::Error as de::Error>::custom(msg));
                }
                map.insert(key.to_owned(), value.to_owned());
            }
            try!(visitor.end());
            Ok(map)
        }
    }

    deserializer.deserialize_map(MapOrKeyValueListVisitor)
}

/// Given a map, deserialize it normally.  But if we have a list of string
/// values, deserialize it as a map keyed with those strings, and with
/// `Default::default()` used as the value.
pub fn deserialize_map_or_default_list<T, D>
    (deserializer: &mut D)
     -> Result<BTreeMap<String, T>, D::Error>
    where T: Default + Deserialize,
          D: Deserializer
{
    /// Declare an internal visitor type to handle our input.
    struct MapOrDefaultListVisitor<T>(PhantomData<T>) where T: Default + Deserialize;

    impl<T: Default + Deserialize> Visitor for MapOrDefaultListVisitor<T> {
        type Value = BTreeMap<String, T>;

        fn visit_map<M>(&mut self, visitor: M) -> Result<Self::Value, M::Error>
            where M: MapVisitor
        {
            let mut mvd = de::value::MapVisitorDeserializer::new(visitor);
            Deserialize::deserialize(&mut mvd)
        }

        fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: SeqVisitor
        {
            let mut map: Self::Value = BTreeMap::new();
            // TODO LOW: Fail with error if values are interpolated.
            while let Some(key) = try!(visitor.visit::<String>()) {
                map.insert(key, Default::default());
            }
            Ok(map)
        }
    }

    deserializer.deserialize(MapOrDefaultListVisitor(PhantomData::<T>))
}

/// Deserialize either list or a single bare string as a list.
pub fn deserialize_item_or_list<T, D>(deserializer: &mut D)
                                      -> Result<Vec<RawOr<T>>, D::Error>
    where T: InterpolatableValue,
          D: Deserializer
{
    /// Our Visitor type, tagged with a 0-size `PhantomData` value so that it
    /// can carry type information.
    struct StringOrListVisitor<T>(PhantomData<T>) where T: InterpolatableValue;

    impl<T> Visitor for StringOrListVisitor<T>
        where T: InterpolatableValue
    {
        type Value = Vec<RawOr<T>>;

        // Handle a single item.
        fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            let v = try!(raw(value).map_err(|err| E::custom(format!("{}", err))));
            Ok(vec![v])
        }

        // Handle a list of items.
        fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: SeqVisitor
        {
            let mut items: Vec<RawOr<T>> = vec![];
            while let Some(item) = try!(visitor.visit::<String>()) {
                let v = try!(raw(item).map_err(|err| {
                    <V::Error as de::Error>::custom(format!("{}", err))
                }));
                items.push(v);
            }
            try!(visitor.end());
            Ok(items)
        }
    }

    deserializer.deserialize(StringOrListVisitor(PhantomData))
}

/// Make sure that the file conforms to a version we can parse.
pub fn check_version<D>(deserializer: &mut D) -> Result<String, D::Error>
    where D: Deserializer
{
    let version = try!(String::deserialize(deserializer));
    if &version != "2" {
        let msg = format!("Can only deserialize docker-compose.yml version 2, found \
                           {}",
                          version);
        return Err(<D::Error as de::Error>::custom(msg));
    }
    Ok(version)
}
