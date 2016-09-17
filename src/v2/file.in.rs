// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A `docker-compose.yml` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    /// The version of the `docker-compose.yml` file format.  Must be 2.
    #[serde(deserialize_with = "check_version")]
    version: String,

    /// The individual services which make up this app.
    pub services: BTreeMap<String, Service>,

    // TODO MED: volumes

    /// The networks used by this app.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub networks: BTreeMap<String, Network>,
}

derive_standard_impls_for!(File, {
    version, services, networks
});

impl File {
    /// Read a file from an input stream containing YAML.
    pub fn read<R>(r: R) -> Result<Self, Error>
        where R: io::Read
    {
        Ok(try!(serde_yaml::from_reader(r)))
    }

    /// Write a file to an output stream as YAML.
    pub fn write<W>(&self, w: &mut W) -> Result<(), Error>
        where W: io::Write
    {
        Ok(try!(serde_yaml::to_writer(w, self)))
    }

    /// Read a file from the specified path.
    pub fn read_from_path<P>(path: P) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        Self::read(try!(fs::File::open(path)))
    }

    /// Write a file to the specified path.
    pub fn write_to_path<P>(&self, path: P) -> Result<(), Error>
        where P: AsRef<Path>
    {
        self.write(&mut try!(fs::File::create(path)))
    }

    /// Inline all our external resources, such as `env_files`, looking up
    /// paths relative to `base`.
    pub fn inline_all(&mut self, base: &Path) -> Result<(), Error> {
        for (_name, service) in self.services.iter_mut() {
            try!(service.inline_all(base));
        }
        Ok(())
    }

    /// Convert this file to a standalone file, with no dependencies on the
    /// current environment or any external files.  This does _not_ lock
    /// down the image versions used in this file.
    pub fn make_standalone(&mut self, base: &Path) -> Result<(), Error> {
        // We need to interpolate first, in case there are environment
        // variables being used to construct the paths to `env_files`
        // entries.
        try!(self.interpolate_all());
        self.inline_all(base)
    }
}

impl Default for File {
    fn default() -> File {
        File {
            version: "2".to_owned(),
            services: Default::default(),
            networks: Default::default(),
        }
    }
}

impl FromStr for File {
    type Err = serde_yaml::Error;

    fn from_str(s: &str) -> Result<File, Self::Err> {
        serde_yaml::from_str(&s)
    }
}

#[test]
fn file_can_be_converted_from_and_to_yaml() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"version": "2"
"#;
    assert_roundtrip!(File, yaml);

    let file: File = serde_yaml::from_str(&yaml).unwrap();
    let foo = file.services.get("foo").unwrap();
    assert_eq!(foo.build.as_ref().unwrap().context, value(Context::new(".")));
}

#[test]
fn file_can_only_load_from_version_2() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"version": "3"
"#;
    assert!(serde_yaml::from_str::<File>(&yaml).is_err());
}
