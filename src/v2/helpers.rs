//! Helper functions and types we use for (de)serialization.  These handle
//! several common, annoying patterns in the `docker-compose.yml` format.

use regex::Regex;
use serde::Error;
use serde::de;
use serde::de::{Deserialize, Deserializer, MapVisitor, SeqVisitor, Visitor};
use serde::ser::{Serialize, Serializer};
use std::collections::BTreeMap;

/// Normalize YAML-format data for comparison purposes.  Used by unit
/// tests.
#[cfg(test)]
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
/// ```rust,ignore
/// struct Example {
///     #[serde(deserialize_with = "deserialize_hash_or_key_value_list")]
///     pub args: BTreeMap<String, String>,
/// }
/// ```
pub fn deserialize_map_or_key_value_list<D>(deserializer: &mut D) ->
    Result<BTreeMap<String, String>, D::Error>
    where D: Deserializer
{
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
                    return Err(<V::Error as Error>::custom(msg));
                }
                map.insert(key, try!(visitor.visit_value::<String>()));
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
                    <V::Error as Error>::custom(msg)
                }));
                let key = caps.at(1).unwrap();
                let value = caps.at(2).unwrap();
                if map.contains_key(key) {
                    let msg = format!("duplicate map key: {}", key);
                    return Err(<V::Error as Error>::custom(msg));
                }
                map.insert(key.to_owned(), value.to_owned());
            }
            try!(visitor.end());
            Ok(map)
        }
    }

    deserializer.deserialize_map(MapOrKeyValueListVisitor)
}

/// A wrapper type which calls `deserialize_map_or_key_value_list`, for
/// those times when we're already manually deserializing the data
/// structure containing this field.  Search the source tree for examples;
/// it's tricky.
#[derive(Debug)]
pub struct MapOrKeyValueList(pub BTreeMap<String, String>);

impl MapOrKeyValueList {
    /// Convert this MapOrKeyValueList into the underlying BTreeMap.
    pub fn into_map(self) -> BTreeMap<String, String> {
        match self {
            MapOrKeyValueList(map) => map
        }
    }
}

impl Deserialize for MapOrKeyValueList {
    fn deserialize<D>(deserializer: &mut D) -> Result<MapOrKeyValueList, D::Error>
        where D: Deserializer
    {
        let map = try!(deserialize_map_or_key_value_list(deserializer));
        Ok(MapOrKeyValueList(map))
    }
}

/// Serialize a list normally, unless it has only a single element, in
/// which case serialize just that element directly.
pub fn serialize_item_or_list<T, S>(value: &[T], serializer: &mut S) ->
    Result<(), S::Error>
    where T: Serialize, S: Serializer
{
    if value.len() == 1 {
        value[0].serialize(serializer)
    } else {
        let mut state = try!(serializer.serialize_seq(Some(value.len())));
        for item in value {
            try!(serializer.serialize_seq_elt(&mut state, item));
        }
        serializer.serialize_seq_end(state)
    }
}

/// Deserialize either list or a single bare string as a list.
pub fn deserialize_string_or_list<D>(deserializer: &mut D) ->
    Result<Vec<String>, D::Error>
    where D: Deserializer
{
    struct StringOrListVisitor;

    impl Visitor for StringOrListVisitor {
        type Value = Vec<String>;

        // Handle a single item.
        fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(vec!(value.to_owned()))
        }

        // Handle a list of items.
        fn visit_seq<V>(&mut self, mut visitor: V) -> Result<Self::Value, V::Error>
            where V: SeqVisitor
        {
            let mut items: Vec<String> = vec!();
            while let Some(item) = try!(visitor.visit::<String>()) {
                items.push(item);
            }
            try!(visitor.end());
            Ok(items)
        }

    }

    deserializer.deserialize(StringOrListVisitor)
}
