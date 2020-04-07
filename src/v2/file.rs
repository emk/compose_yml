// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A `docker-compose.yml` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    /// The version of the `docker-compose.yml` file format.  Must be 2.
    pub version: String,

    /// The individual services which make up this app.
    pub services: BTreeMap<String, Service>,

    /// Named volumes used by this app.
    ///
    /// TODO MED: Can we parse just volume names followed by a colon?
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_struct_or_null")]
    pub volumes: BTreeMap<String, Volume>,

    /// The networks used by this app.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_struct_or_null")]
    pub networks: BTreeMap<String, Network>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(File, {
    version, services, volumes, networks, _hidden
});

impl File {
    /// Read a file from an input stream containing YAML.
    pub fn read<R>(r: R) -> Result<Self>
        where R: io::Read
    {
        let file = serde_yaml::from_reader(r)?;
        validate_file(&file)?;
        Ok(file)
    }

    /// Write a file to an output stream as YAML.
    pub fn write<W>(&self, w: &mut W) -> Result<()>
        where W: io::Write
    {
        validate_file(self)?;
        Ok(serde_yaml::to_writer(w, self)?)
    }

    /// Read a file from the specified path.
    pub fn read_from_path<P>(path: P) -> Result<Self>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let f = fs::File::open(path).map_err(|err| Error::read_file(path.to_owned(), err))?;
        Self::read(io::BufReader::new(f)).map_err(|err| Error::read_file(path.to_owned(), err))
    }

    /// Write a file to the specified path.
    pub fn write_to_path<P>(&self, path: P) -> Result<()>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let f = fs::File::create(path).map_err(|err| Error::write_file(path.to_owned(), err))?;
        self.write(&mut io::BufWriter::new(f)).map_err(|err| Error::write_file(path.to_owned(), err))
    }

    /// Inline all our external resources, such as `env_files`, looking up
    /// paths relative to `base`.
    pub fn inline_all(&mut self, base: &Path) -> Result<()> {
        for service in self.services.values_mut() {
            service.inline_all(base)?;
        }
        Ok(())
    }

    /// Convert this file to a standalone file, with no dependencies on the
    /// current environment or any external files.  This does _not_ lock
    /// down the image versions used in this file.
    pub fn make_standalone(&mut self, base: &Path) -> Result<()> {
        // We need to interpolate first, in case there are environment
        // variables being used to construct the paths to `env_files`
        // entries.
        self.interpolate_all()?;
        self.inline_all(base)
    }
}

impl Default for File {
    fn default() -> File {
        File {
            version: "2.4".to_owned(),
            services: Default::default(),
            volumes: Default::default(),
            networks: Default::default(),
            _hidden: (),
        }
    }
}

impl FromStr for File {
    type Err = Error;

    fn from_str(s: &str) -> Result<File> {
        Self::read(io::Cursor::new(s))
    }
}

#[test]
#[cfg_attr(feature="clippy", allow(blacklisted_name))]
fn file_can_be_converted_from_and_to_yaml_version_2() {
    let yaml = r#"---
services:
  foo:
    build: .
version: "2"
volumes:
  db:
    external: true
"#;
    assert_roundtrip!(File, yaml);

    let file = File::from_str(&yaml).unwrap();
    let foo = file.services.get("foo").unwrap();
    assert_eq!(foo.build.as_ref().unwrap().context, value(Context::new(".")));
}

#[test]
fn file_can_be_converted_from_and_to_yaml_version_2_1() {
    let yaml = r#"---
services:
  foo:
    build: .
version: "2.1"
volumes:
  db:
    external: true
"#;
    assert_roundtrip!(File, yaml);
}

#[test]
fn file_allows_null_volumes_and_networks() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"networks":
  "frontend":
  "internal":
"version": "2"
"volumes":
  "bar":
  "foo":
"#;
    let file = File::from_str(&yaml).unwrap();
    assert_eq!(file.volumes.len(), 2);
    assert_eq!(file.networks.len(), 2);
}

#[test]
fn file_checks_version_number() {
    let yaml = r#"---
"services":
  "foo":
    "build": "."
"version": "100"
"#;
    assert!(File::from_str(&yaml).is_err());
}

// TODO: Disabled pending https://github.com/emk/compose_yml/issues/11
#[test]
#[ignore]
fn file_validates_against_schema() {
    let yaml = r#"---
"version": "2"
"services":
  # An invalid service name:
  "foo!":
    "build": "."
"#;
    assert!(File::from_str(&yaml).is_err());
}
