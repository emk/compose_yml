// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Information on how to build a Docker image.
#[derive(Debug, Serialize)]
pub struct Build {
    /// The source directory to use for this build.
    pub context: Context,

    /// The name of an alternate `Dockerfile` to use.
    pub dockerfile: Option<String>,
}

// This hideous deserializer handles the fact that `build:` can be
// serialized as either a bare context string, or as a structure with
// multiple nested keys.
impl Deserialize for Build {
    fn deserialize<D>(deserializer: &mut D) -> Result<Build, D::Error>
        where D: Deserializer
    {
        // We create a `Visitor` type, with one method for each data type
        // we support.  The deserializer will call the method corresponding
        // to the data that's actually in the file.
        struct BuildVisitor;

        impl Visitor for BuildVisitor {
            type Value = Build;

            // The deserializer found a string, so handle it.
            fn visit_str<E>(&mut self, value: &str) -> Result<Build, E>
                where E: de::Error
            {
                Ok(Build {
                    context: Context::new(value),
                    dockerfile: None,
                })
            }

            // The deserializer found a key/value map.  We'll need to
            // extract the keys and values one at a time and turn them into
            // an object.
            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Build, V::Error>
                where V: MapVisitor
            {
                let mut context: Option<Context> = None;
                let mut dockerfile: Option<String> = None;

                while let Some(key) = try!(visitor.visit_key::<String>()) {
                    match key.as_ref() {
                        "context" => {
                            if context.is_some() {
                                return Err(<V::Error as Error>::duplicate_field("context"));
                            }
                            context = Some(try!(visitor.visit_value()));
                        }
                        "dockerfile" => {
                            if dockerfile.is_some() {
                                return Err(<V::Error as Error>::duplicate_field("dockerfile"));
                            }
                            dockerfile = Some(try!(visitor.visit_value()));
                        }
                        name => {
                            return Err(<V::Error as Error>::unknown_field(name));
                        }
                    }
                }
                try!(visitor.end());
                let context = match context {
                    Some(context) => context,
                    None => try!(visitor.missing_field("context")),
                };
                Ok(Build {
                    context: context,
                    dockerfile: dockerfile,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["context"];
        deserializer.deserialize_struct("Build", FIELDS, BuildVisitor)
    }
}

#[test]
fn build_may_be_a_bare_string() {
    let build: Build = serde_yaml::from_str("---\n\".\"").unwrap();
    assert_eq!(build.context, Context::new("."));
    assert_eq!(build.dockerfile, None);
}

#[test]
fn build_may_be_a_struct() {
    let yaml = "---
context: \".\"
dockerfile: \"Dockerfile\"
";
    let build: Build = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(build.context, Context::new("."));
    assert_eq!(build.dockerfile, Some("Dockerfile".to_owned()));
}
