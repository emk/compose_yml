//! Special enumeration types with serialization support and string
//! arguments for some values.

use regex::Regex;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use super::helpers::*;

/// This big, bad macro is in charge of implementing serializable enums
/// with entries like:
///
/// ```text
/// bridge
/// host
/// none
/// service:NAME
/// container:NAME
/// ```
///
/// Most of the values are simple strings, but a few values have arguments.
/// There are a lot of these enumerations in the Docker API, and it takes a
/// fair bit of boilerplate to serialize and deserialize them all in a
/// type-safe way.  So instead, we define a monster code-generation macro
/// which pushes Rust's stable macro system pretty much to its limit.
///
/// Here's a simplified example of what it looks like:
///
/// ```
/// mode_enum! {
///     /// How should we configure the container's networking?
///     #[derive(Debug, PartialEq, Eq)]
///     pub enum SimplifiedNetworkMode {
///         /// Use the standard Docker networking bridge.
///         ("bridge") => Bridge,
///         /// Use the host's network interface directly.
///         ("host") => Host
///     ;
///         /// Use the networking namespace associated with the named service.
///         ("service") => Service(String)
///     }
/// }
/// ```
///
/// Note the syntactic oddities:
///
/// 1. All "simple" entries with no arguments go before the semi-colon.
/// 2. All "complex" entries with an argument go after the semi-colon.
/// 3. Commas are always used as separators here and you can't have a
///    trailing comma.  Blame Rust's macro system.
macro_rules! mode_enum {
    (// This pattern matches zero or more doc comments and metadata
     // attributes.
     $(#[$flag:meta])*
     pub enum $name:ident {
        // This pattern matches a list of enum values with no args.
        $(
            $(#[$flag0:meta])*
            ($tag0:expr) => $item0:ident
        ),*
    // Mandatory separator to avoid the need for lookahead to tell where
    // simple args stop and complex ones start.
    ;
        // This pattern matches a list of enum values with single args
        // of various types.
        $(
            $(#[$flag1:meta])*
            ($tag1:expr) => $item1:ident($arg:ty)
        ),*
    }) => {
        $(#[$flag])*
        pub enum $name {
            // Insert all our enum definitions here.
            $(
                $(#[$flag0])*
                $item0,
            )*
            $(
                $(#[$flag1])*
                $item1($arg),
            )*
        }

        // Set up serialization to strings.
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                match self {
                    $( &$name::$item0 => write!(f, $tag0), )*
                    $( &$name::$item1(ref name) =>
                           write!(f, "{}:{}", $tag1, name), )*
                }
            }
        }

        impl_serialize_to_string!($name);

        // Set up deserialization from strings.
        impl FromStr for $name {
            type Err = InvalidValueError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                lazy_static! {
                    static ref COMPOUND: Regex =
                        Regex::new("^([a-z]+):(.+)$").unwrap();
                }

                match s {
                    $( $tag0 => Ok($name::$item0), )*
                    _ => {
                        let caps = try!(COMPOUND.captures(s).ok_or_else(|| {
                            InvalidValueError::new(stringify!($name), s)
                        }));
                        let valstr = caps.at(2).unwrap();
                        match caps.at(1).unwrap() {
                            $( $tag1 => {
                               let value = try!(FromStr::from_str(valstr).map_err(|_| {
                                   InvalidValueError::new(stringify!($name),
                                                          valstr)
                               }));
                               Ok($name::$item1(value))
                            })*
                            _ => Err(InvalidValueError::new(stringify!($name), s))
                        }
                    }
                }
            }
        }

        impl_deserialize_from_str!($name);
    }
}

mode_enum! {
    /// How should we configure the container's networking?
    #[derive(Debug, PartialEq, Eq)]
    pub enum NetworkMode {
        /// Use the standard Docker networking bridge.
        ("bridge") => Bridge,
        /// Use the host's network interface directly.
        ("host") => Host,
        /// Disable networking in the container.
        ("none") => None
    ;
        /// Use the networking namespace associated with the named service.
        ("service") => Service(String),
        /// Use the networking namespace associated with the named container.
        ("container") => Container(String)
    }
}

#[test]
fn network_mode_has_a_string_representation() {
    let pairs = vec!(
        (NetworkMode::Bridge, "bridge"),
        (NetworkMode::Host, "host"),
        (NetworkMode::None, "none"),
        (NetworkMode::Service("foo".to_owned()), "service:foo"),
        (NetworkMode::Container("foo".to_owned()), "container:foo"),
    );
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, NetworkMode::from_str(s).unwrap());
    }
}

mode_enum! {
    /// What process ID namespace should we use?
    #[derive(Debug, PartialEq, Eq)]
    pub enum PidMode {
        /// Use the host's PID namespace.
        ("host") => Host
    ;
        // Use another service's namespace.  This _should_ exist, but it's
        // not documented.  Feel free to uncomment and try.
        //("service") => Service(String),
        /// Use the named container's PID namespace.
        ("container") => Container(String)
    }
}

mode_enum! {
    /// What IPC namespace should we use for our container?
    #[derive(Debug, PartialEq, Eq)]
    pub enum IpcMode {
        /// Use the host's IPC namespace.
        ("host") => Host
    ;
        // Use another service's namespace.  This _should_ exist, but it's
        // not documented.  Feel free to uncomment and try.
        //("service") => Service(String),
        /// Use the named container's IPC namespace.
        ("container") => Container(String)
    }
}
