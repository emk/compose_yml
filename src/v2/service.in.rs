// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A service which will be managed by `docker-compose`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    /// How to build an image for this service.
    pub build: Option<Build>,
}
