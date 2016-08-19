// To get better error messages for this file, build it using the nightly
// release of Rust:
//
// ```sh
// rustup toolchain install nightly
// rustup run nightly cargo build --no-default-features --features unstable
// ```


/// A `docker-compose.yml` file.
#[derive(Serialize, Deserialize, Debug)]
struct File {
    services: HashMap<String, Service>,
}

#[test]
fn file_can_be_converted_from_and_to_yaml() {
    let yaml = "---
services:
  foo:
    build:
      context: .
";
    let file: File = serde_yaml::from_str(&yaml).unwrap();
    let foo = file.services.get("foo").unwrap();
    assert_eq!(foo.build.as_ref().unwrap().context, ".");

    serde_yaml::to_string(&file).unwrap();
}


/// A service which will be managed by `docker-compose`.
#[derive(Serialize, Deserialize, Debug)]
struct Service {
    build: Option<Build>,
}

/// Information on how to build
#[derive(Serialize, Deserialize, Debug)]
struct Build {
    context: String,
}
