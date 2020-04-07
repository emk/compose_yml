//! Helper functions and types we use for (de)serialization.  These handle
//! several common, annoying patterns in the `docker-compose.yml` format.

use regex::Regex;
use serde::de;
use serde::de::{
    Deserialize, DeserializeOwned, Deserializer, MapAccess, SeqAccess, Visitor,
};
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use super::interpolation::{raw, InterpolatableValue, RawOr};

/// Test whether a value is false.  Used to determine when to serialize
/// things.
pub fn is_false(b: &bool) -> bool {
    !b
}

/// We use this when the format wants a `String`, but has support for
/// converting several other types.  Mostly this is so that users can
/// write `ENV_VAR: 1`, and not get an error about using `1` instead of
/// `"1"`, and for compatibility with `docker-compose`.
struct ToStringVisitor;

impl<'de> Visitor<'de> for ToStringVisitor {
    type Value = String;

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(format!("{}", v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(format!("{}", v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(format!("{}", v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.to_owned())
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a value which can be converted to a string")
    }
}

/// A wrapper type which uses `ToStringVisitor` to deserialize a value,
/// converting many scalar types to a string.
struct ConvertToString(String);

impl<'de> Deserialize<'de> for ConvertToString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_string(ToStringVisitor)
            .map(ConvertToString)
    }
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
///     pub args: BTreeMap<String, RawOr<String>>,
/// }
/// ```
pub fn deserialize_map_or_key_value_list<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, RawOr<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    /// Declare an internal visitor type to handle our input.
    struct MapOrKeyValueListVisitor;

    impl<'de> Visitor<'de> for MapOrKeyValueListVisitor {
        type Value = BTreeMap<String, RawOr<String>>;

        // We have a real map.
        fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: MapAccess<'de>,
        {
            let mut map: BTreeMap<String, RawOr<String>> = BTreeMap::new();
            while let Some(key) = visitor.next_key::<String>()? {
                if map.contains_key(&key) {
                    let msg = format!("duplicate map key: {}", &key);
                    return Err(<V::Error as de::Error>::custom(msg));
                }
                let ConvertToString(val) = visitor.next_value::<ConvertToString>()?;
                let raw_or_value = raw(val)
                    .map_err(|e| <V::Error as de::Error>::custom(format!("{}", e)))?;
                map.insert(key, raw_or_value);
            }
            Ok(map)
        }

        // We have a key/value list.  Yuck.
        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            lazy_static! {
                // Match a key/value pair.
                static ref KEY_VALUE: Regex =
                    Regex::new("^([^=]+)=(.*)$").unwrap();
            }

            let mut map: BTreeMap<String, RawOr<String>> = BTreeMap::new();
            while let Some(key_value) = visitor.next_element::<String>()? {
                let caps = KEY_VALUE.captures(&key_value).ok_or_else(|| {
                    let msg = format!("expected KEY=value, got: <{}>", &key_value);
                    <V::Error as de::Error>::custom(msg)
                })?;
                let key = caps.get(1).unwrap().as_str();
                let value = caps.get(2).unwrap().as_str();
                if map.contains_key(key) {
                    let msg = format!("duplicate map key: {}", key);
                    return Err(<V::Error as de::Error>::custom(msg));
                }
                let raw_or_value = raw(value.to_owned())
                    .map_err(|e| <V::Error as de::Error>::custom(format!("{}", e)))?;
                map.insert(key.to_owned(), raw_or_value);
            }
            Ok(map)
        }

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a map or a key/value list")
        }
    }

    deserializer.deserialize_map(MapOrKeyValueListVisitor)
}

/// Given a map, deserialize it normally.  But if we have a list of string
/// values, deserialize it as a map keyed with those strings, and with
/// `Default::default()` used as the value.
pub fn deserialize_map_or_default_list<'de, T, D>(
    deserializer: D,
) -> Result<BTreeMap<String, T>, D::Error>
where
    T: Default + DeserializeOwned,
    D: Deserializer<'de>,
{
    /// Declare an internal visitor type to handle our input.
    struct MapOrDefaultListVisitor<T>(PhantomData<T>)
    where
        T: Default + DeserializeOwned;

    impl<'de, T: Default + DeserializeOwned> Visitor<'de> for MapOrDefaultListVisitor<T> {
        type Value = BTreeMap<String, T>;

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mvd = de::value::MapAccessDeserializer::new(visitor);
            Deserialize::deserialize(mvd)
        }

        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map: Self::Value = BTreeMap::new();
            // TODO LOW: Fail with error if values are interpolated.
            while let Some(key) = visitor.next_element::<String>()? {
                map.insert(key, Default::default());
            }
            Ok(map)
        }

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a map or a list of strings")
        }
    }

    deserializer.deserialize_map(MapOrDefaultListVisitor(PhantomData::<T>))
}

/// Deserialize either list or a single bare string as a list.
pub fn deserialize_item_or_list<'de, T, D>(
    deserializer: D,
) -> Result<Vec<RawOr<T>>, D::Error>
where
    T: InterpolatableValue,
    D: Deserializer<'de>,
{
    /// Our Visitor type, tagged with a 0-size `PhantomData` value so that it
    /// can carry type information.
    struct StringOrListVisitor<T>(PhantomData<T>)
    where
        T: InterpolatableValue;

    impl<'de, T> Visitor<'de> for StringOrListVisitor<T>
    where
        T: InterpolatableValue,
    {
        type Value = Vec<RawOr<T>>;

        // Handle a single item.
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let v = raw(value).map_err(|err| E::custom(format!("{}", err)))?;
            Ok(vec![v])
        }

        // Handle a list of items.
        fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut items: Vec<RawOr<T>> = vec![];
            while let Some(item) = visitor.next_element::<String>()? {
                let v = raw(item).map_err(|err| {
                    <V::Error as de::Error>::custom(format!("{}", err))
                })?;
                items.push(v);
            }
            Ok(items)
        }

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a string or a list of strings")
        }
    }

    deserializer.deserialize_seq(StringOrListVisitor(PhantomData))
}

/// Deserialize either list or a single bare string as a list.
pub fn deserialize_map_struct_or_null<'de, T, D>(
    deserializer: D,
) -> Result<BTreeMap<String, T>, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    let with_nulls: BTreeMap<String, Option<T>> =
        Deserialize::deserialize(deserializer)?;
    let mut result = BTreeMap::new();
    for (k, v) in with_nulls {
        result.insert(k, v.unwrap_or_default());
    }
    Ok(result)
}
