//! Support for validating a `docker-compose.yml` file against the official
//! schema.

use serde::Serialize;
use serde_json;
use std::ops::Deref;
use valico;

use errors::*;
use super::File;

/// Schema for `docker-compose.yml` version 2.0.
const COMPOSE_2_0_SCHEMA_STR: &'static str = include_str!("config_schema_v2.0.json");

/// Schema for `docker-compose.yml` version 2.1.
const COMPOSE_2_1_SCHEMA_STR: &'static str = include_str!("config_schema_v2.1.json");

/// Load and parse a built-in JSON file, panicking if it contains invalid
/// JSON.
fn load_schema_json(json: &'static str) -> serde_json::Value {
    match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => panic!("cannot parse built-in schema: {}", err),
    }
}

lazy_static! {
    /// Parsed schema for `docker-compose.yml` version 2.0.
    static ref COMPOSE_2_0_SCHEMA: serde_json::Value =
        load_schema_json(COMPOSE_2_0_SCHEMA_STR);

    /// Parsed schema for `docker-compose.yml` version 2.1.
    static ref COMPOSE_2_1_SCHEMA: serde_json::Value =
        load_schema_json(COMPOSE_2_1_SCHEMA_STR);
}

/// Validate a `File` against the official JSON schema provided by
/// `docker-compose`.
pub fn validate_file(file: &File) -> Result<()> {
    let schema_value = match &file.version[..] {
        "2.0" => COMPOSE_2_0_SCHEMA.deref(),
        "2.1" => COMPOSE_2_1_SCHEMA.deref(),
        vers => return Err(ErrorKind::UnsupportedVersion(vers.to_owned()).into()),
    };

    let mut scope = valico::json_schema::Scope::new();
    let schema = match scope.compile_and_return(schema_value.clone(), false) {
        Ok(schema) => schema,
        Err(err) => panic!("cannot parse built-in schema: {:?}", err),
    };

    let mut serializer = serde_json::value::Serializer::new();
    file.serialize(&mut serializer)
        .chain_err(|| ErrorKind::ValidationFailed)?;
    let value = serializer.unwrap();
    let validation_state = schema.validate(&value);
    if validation_state.is_strictly_valid() {
        Ok(())
    } else {
        let res: Result<()> = Err(validation_state.into());
        res.chain_err(|| ErrorKind::ValidationFailed)
    }
}
