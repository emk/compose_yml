// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Settings for performing health checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// The command to run to perform the health check.
    pub test: CommandLine,

    /// Interval between health checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,

    /// How long health checks are retried.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,

    /// Number of times to retry health checks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,

    /// Time to wait before counting any failed checks against total 
    /// number of retries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,


    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(HealthCheck, {
    test, interval, timeout, retries, start_period, _hidden
});