/// A limit imposed on the resources used by a process.
///
/// We use `#[serde(untagged)]` to parse the two different variants of this enum
/// automatically, which makes it harder to use certain `cage` features but
/// which makes it easy to implement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, untagged)]
#[allow(missing_copy_implementations)]
pub enum Ulimit {
    /// A single limit.
    Single(i64),
    /// A pair of soft and hard limits.
    Pair {
        /// This limit can be changed by the process itself.
        soft: i64,
        /// This limit can only be changed by root.
        hard: i64,
    }
}

impl InterpolateAll for Ulimit {}
impl MergeOverride for Ulimit {}

#[test]
fn ulimit_single() {
    let yaml = r#"---
1024
"#;
    assert_roundtrip!(Ulimit, yaml);
}

#[test]
fn ulimit_pair() {
    let yaml = r#"---
soft: 1024
hard: 2048
"#;
    assert_roundtrip!(Ulimit, yaml);
}
