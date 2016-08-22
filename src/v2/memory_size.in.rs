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

impl SimpleSerializeDeserialize for MemorySize {
    fn to_string(&self) -> Result<String, InvalidValueError> {
        let bytes = self.to_bytes();
        if bytes == 0 {
            // Just print 0 without any units, because anything else looks
            // weird.
            Ok("0".to_owned())
        } else if bytes % (1024*1024*1024) == 0 {
            Ok(format!("{}g", bytes / (1024*1024*1024)))
        } else if bytes % (1024*1024) == 0 {
            Ok(format!("{}m", bytes / (1024*1024)))
        } else if bytes % 1024 == 0 {
            Ok(format!("{}k", bytes / 1024))
        } else {
            // TODO: Verify `b` is the default specifier.
            Ok(format!("{}", bytes))
        }
    }

    fn from_str(s: &str) -> Result<Self, InvalidValueError> {
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
            _ => panic!("Unexpected error parsing MemorySize <{}>", s),
        }
    }
}

// Implement Serialize and Deserialize using a macro, to work around
// restrictions in Rust's trait system.
impl_simple_serialize_deserialize!(MemorySize);

#[test]
fn memory_size_supports_simple_serialization() {
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
        assert_eq!(mem_sz.to_string().unwrap(), s);
        assert_eq!(mem_sz, MemorySize::from_str(s).unwrap());
    }

    assert_eq!(MemorySize::bytes(10), MemorySize::from_str("10b").unwrap());
}
