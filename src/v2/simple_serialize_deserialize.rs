//! Support for types which serialize and deserialize as simple strings.

use std::error;
use std::fmt;

/// An error parsing a string in a Dockerfile.
#[derive(Debug)]
pub struct InvalidValueError {
    wanted: String,
    input: String,
}

impl InvalidValueError {
    pub fn new(wanted: &str, input: &str) -> InvalidValueError {
        InvalidValueError {
            wanted: wanted.to_owned(),
            input: input.to_owned(),
        }
    }
}

impl fmt::Display for InvalidValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Invalid {}: <{}>", &self.wanted, &self.input)
    }
}

impl error::Error for InvalidValueError {
    fn description(&self) -> &str {
        "Invalid value"
    }
}

/// This trait provides an easier way to implement `Serialize` and
/// `Deserialize` for types that are represented as strings.
pub trait SimpleSerializeDeserialize: Sized {
    /// Serialize an object of this type into a string.
    fn to_string(&self) -> Result<String, InvalidValueError>;

    /// Parse a string into an object of this type.
    fn from_str(s: &str) -> Result<Self, InvalidValueError>;
}

/// Provide implementations of `Serialize` and `Deserialize` for all
/// `SimpleSerializeDeserialize` types.  See [this
/// discussion][external_traits] for an explanation of what's going on
/// here, and why we need to resort to a macro.
///
/// [external_traits]: https://www.reddit.com/r/rust/comments/3709tl/implementing_external_trait_for_types/
#[macro_export]
macro_rules! impl_simple_serialize_deserialize {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: Serializer
            {
                match &self.to_string() {
                    &Ok(ref s) => serializer.serialize_str(s),
                    &Err(ref err) => Err(ser::Error::custom(format!("{}", err)))
                }
            }
        }

        impl Deserialize for $ty {
            fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
                where D: Deserializer
            {
                let string = try!(String::deserialize(deserializer));
                Self::from_str(&string).map_err(|err| {
                    de::Error::custom(format!("{}", err))
                })
            }
        }
    }
}
