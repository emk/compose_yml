// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

macro_rules! mode_enum {
    ($(#[$flag:meta])*
     pub enum $name:ident {
        $(
            $(#[$flag0:meta])*
            ($tag0:expr) => $item0:ident
        ),*
    ;
        $(
            $(#[$flag1:meta])*
            ($tag1:expr) => $item1:ident(String)
        ),*
    }) => {
        $(#[$flag])*
        pub enum $name {
            $(
                $(#[$flag0])*
                $item0,
            )*
            $(
                $(#[$flag1])*
                $item1(String),
            )*
        }

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
                        let name = caps.at(2).unwrap().to_owned();
                        match caps.at(1).unwrap() {
                            $( $tag1 => Ok($name::$item1(name)), )*
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
