// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Information on how to build a Docker image.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    /// The source directory to use for this build.
    pub context: RawOr<Context>,

    /// The name of an alternate `Dockerfile` to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dockerfile: Option<RawOr<String>>,

    /// Build arguments.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_value_list")]
    pub args: BTreeMap<String, RawOr<String>>,

    /// The FROM target at which to stop building
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<RawOr<String>>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(Build, {
    context, dockerfile, args, target, _hidden
});

impl Build {
    /// Create a new build from just `Context`.  To override other fields, you
    /// can use struct notation.
    ///
    /// ```
    /// use compose_yml::v2 as dc;
    ///
    /// dc::Build::new(dc::Context::new("app"));
    ///
    /// dc::Build {
    ///   dockerfile: Some(dc::escape("Dockerfile-alt").unwrap()),
    ///   ..dc::Build::new(dc::Context::new("app"))
    /// };
    /// ```
    pub fn new<C: Into<Context>>(ctx: C) -> Self {
        Build {
            context: value(ctx.into()),
            dockerfile: Default::default(),
            args: Default::default(),
            target: Default::default(),
            _hidden: (),
        }
    }
}

impl FromStr for Build {
    // We never return an error, so specify `Void` as our error type.
    type Err = Void;

    fn from_str(s: &str) -> result::Result<Self, Self::Err> {
        Ok(Build::new(Context::new(s)))
    }
}

impl SerializeStringOrStruct for Build {
    fn serialize_string_or_struct<S>(&self, serializer: S) ->
        result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        if self.dockerfile.is_none() && self.args.is_empty() {
            self.context.serialize(serializer)
        } else {
            self.serialize(serializer)
        }
    }
}

#[test]
fn build_has_a_string_representation() {
    let build: Build = Build::from_str(".").unwrap();
    assert_eq!(build.context, value(Context::new(".")));
    assert_eq!(build.dockerfile, None);
    assert_eq!(build.args, Default::default());
}

#[test]
fn build_may_be_a_struct() {
    let yaml = r#"---
args:
  key: value
context: .
dockerfile: Dockerfile
"#;
    assert_roundtrip!(Build, yaml);

    let build: Build = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(build.context, value(Context::new(".")));
    assert_eq!(build.dockerfile, Some(value("Dockerfile".to_owned())));
    assert_eq!(build.args.get("key").expect("wanted key 'key'").value().unwrap(),
               "value");
}

#[test]
fn args_support_stringification_and_interpolation() {
    let yaml = r#"---
context: .
args:
  bool: true
  float: 1.5
  int: 1
  interp: $FOO
"#;
    let build: Build = serde_yaml::from_str(yaml).unwrap();

    // Check type conversion.
    assert_eq!(build.args.get("bool").expect("wanted key 'bool'").value().unwrap(),
               "true");
    assert_eq!(build.args.get("float").expect("wanted key 'float'").value().unwrap(),
               "1.5");
    assert_eq!(build.args.get("int").expect("wanted key 'int'").value().unwrap(),
               "1");

    // Check interpolation.
    let mut interp: RawOr<String> =
        build.args.get("interp").expect("wanted key 'interp'").to_owned();
    env::set_var("FOO", "foo");
    let env = OsEnvironment::new();
    assert_eq!(interp.interpolate_env(&env).unwrap(), "foo")
}

#[test]
fn build_args_may_be_a_key_value_list() {
    let yaml = "---
context: .
args:
  - key=value
";
    let build: Build = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(build.args.get("key").expect("should have key").value().unwrap(),
               "value");
}

// TODO MED: Implement valueless keys.
//
// args:
//   - buildno
//   - password
