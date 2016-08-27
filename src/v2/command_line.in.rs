// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A command line to be executed by Docker.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CommandLine {
    /// A command-line specified as unparsed shell code.
    ShellCode(String),

    /// A pre-parsed command-line.  This may actually be empty for fields
    /// like `command`, so we don't try to enforce a minimal length, even
    /// if other fields like `entrypoint` supposedly want at least one
    /// entry.
    Parsed(Vec<String>),
}

impl Serialize for CommandLine {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        match self {
            &CommandLine::ShellCode(ref s) => serializer.serialize_str(s),
            &CommandLine::Parsed(ref l) => l.serialize(serializer),
        }
    }
}

impl Deserialize for CommandLine {
    fn deserialize<D>(deserializer: &mut D) -> Result<CommandLine, D::Error>
        where D: Deserializer
    {
        struct CommandLineVisitor;

        impl Visitor for CommandLineVisitor {
            type Value = CommandLine;
        
            // The deserializer found a string, so handle it.
            fn visit_str<E>(&mut self, value: &str) -> Result<CommandLine, E>
                where E: de::Error
            {
                Ok(CommandLine::ShellCode(value.to_owned()))
            }

            // The deserializer found a sequence.
            fn visit_seq<V>(&mut self, mut visitor: V) ->
                Result<Self::Value, V::Error>
                where V: SeqVisitor
            {
                let mut args: Vec<String> = vec!();
                while let Some(arg) = try!(visitor.visit::<String>()) {
                    args.push(arg);
                }
                try!(visitor.end());
                Ok(CommandLine::Parsed(args))
            }
        }

        deserializer.deserialize(CommandLineVisitor)
    }
}

#[test]
fn command_line_may_be_shell_code() {
    let yaml = r#"---
"ls /"
"#;
    assert_roundtrip!(CommandLine, yaml);
}

#[test]
fn command_line_may_be_parsed() {
    let yaml = r#"---
- "ls"
- "/"
"#;
    assert_roundtrip!(CommandLine, yaml);
}
