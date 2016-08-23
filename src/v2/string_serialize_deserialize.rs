//! Support for types which serialize and deserialize as simple strings.

/// Provide implementations of `Serialize` for a `ToString` type.  See
/// [this discussion][external_traits] for an explanation of what's going
/// on here, and why we need to resort to a macro.
///
/// Note that you shouldn't actually implement `ToString` for your types.
/// You should implement `Display` and you'll get a default implementation
/// of `ToString` for free, and this what the Rust documentation
/// recommends.
///
/// [external_traits]: https://www.reddit.com/r/rust/comments/3709tl/implementing_external_trait_for_types/
macro_rules! impl_serialize_to_string {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
                where S: Serializer
            {
                serializer.serialize_str(&self.to_string())
            }
        }
    }
}

/// Provide an implementation of `Deserialize` for a `FromString` type.
/// See `impl_serialize_to_string` for an explanation of why we do this
/// with a macro.
macro_rules! impl_deserialize_from_str {
    ($ty:ty) => {
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
