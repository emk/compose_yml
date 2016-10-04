// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// The size of a block of memory. This can be serialized as a
/// Docker-compatible size string using specifiers like `k`, `m` and `g`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MemorySize(usize);

impl MemorySize {
    /// Create a `MemorySize` from a size in bytes.
    pub fn bytes(bytes: usize) -> MemorySize {
        MemorySize(bytes)
    }

    /// Create from a size in kilobytes.
    pub fn kb(kb: usize) -> MemorySize {
        MemorySize(kb * 1024)
    }

    /// Create from a size in megabytes.
    pub fn mb(mb: usize) -> MemorySize {
        MemorySize(mb * 1024 * 1024)
    }

    /// Create from a size in gigabytes.
    pub fn gb(gb: usize) -> MemorySize {
        MemorySize(gb * 1024 * 1024 * 1024)
    }

    /// Convert to a size in bytes.
    pub fn to_bytes(self) -> usize {
        match self {
            MemorySize(bytes) => bytes,
        }
    }
}

impl_interpolatable_value!(MemorySize);

impl fmt::Display for MemorySize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.to_bytes();
        if bytes == 0 {
            // Just print 0 without any units, because anything else looks
            // weird.
            write!(f, "0")
        } else if bytes % (1024*1024*1024) == 0 {
            write!(f, "{}g", bytes / (1024*1024*1024))
        } else if bytes % (1024*1024) == 0 {
            write!(f, "{}m", bytes / (1024*1024))
        } else if bytes % 1024 == 0 {
            write!(f, "{}k", bytes / 1024)
        } else {
            // `b` is the default specifier, so don't print it.
            write!(f, "{}", bytes)
        }
    }
}

impl FromStr for MemorySize {
    type Err = InvalidValueError;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        lazy_static! {
            static ref MEM_SIZE: Regex =
                Regex::new("^([0-9]+)([bkmg])?$").unwrap();
        }
        let caps = try!(MEM_SIZE.captures(s).ok_or_else(|| {
            InvalidValueError::new("memory size", s)
        }));
        let value: usize = caps.at(1).unwrap().parse().unwrap();
        match caps.at(2) {
            None | Some("b") => Ok(MemorySize::bytes(value)),
            Some("k") => Ok(MemorySize::kb(value)),
            Some("m") => Ok(MemorySize::mb(value)),
            Some("g") => Ok(MemorySize::gb(value)),
            _ => unreachable!("Unexpected error parsing MemorySize <{}>", s),
        }
    }
}

#[test]
fn memory_size_supports_string_serialization() {
    let pairs = vec!(
        (MemorySize::bytes(0), "0"),
        (MemorySize::bytes(1), "1"),
        (MemorySize::bytes(1023), "1023"),
        (MemorySize::bytes(1024), "1k"),
        (MemorySize::kb(1), "1k"),
        (MemorySize::bytes(1025), "1025"),
        (MemorySize::mb(1), "1m"),
        (MemorySize::gb(1), "1g"),
    );
    for (mem_sz, s) in pairs {
        assert_eq!(mem_sz.to_string(), s);
        assert_eq!(mem_sz, MemorySize::from_str(s).unwrap());
    }

    assert_eq!(MemorySize::bytes(10), MemorySize::from_str("10b").unwrap());
}
