// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A command line to be executed by Docker.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CommandLine {
    /// A command-line specified as unparsed shell code.
    ShellCode(RawOr<String>),

    /// A pre-parsed command-line.  This may actually be empty for fields
    /// like `command`, so we don't try to enforce a minimal length, even
    /// if other fields like `entrypoint` supposedly want at least one
    /// entry.
    Parsed(Vec<RawOr<String>>),
}

impl MergeOverride for CommandLine {}
impl InterpolateAll for CommandLine {}

#[test]
fn command_line_may_be_shell_code() {
    let yaml = r#"---
ls $DIR
"#;
    assert_roundtrip!(CommandLine, yaml);
}

#[test]
fn command_line_may_be_parsed() {
    let yaml = r#"---
- ls
- /
"#;
    assert_roundtrip!(CommandLine, yaml);
}
