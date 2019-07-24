// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Service condition.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ServiceCondition {
    /// This service must be healthy.
    ServiceHealthy,
    /// This service must be started.
    ServiceStarted,
}

impl_interpolatable_value!(ServiceCondition);

impl fmt::Display for ServiceCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ServiceCondition::ServiceHealthy => write!(f, "service_healthy"),
            &ServiceCondition::ServiceStarted => write!(f, "service_started"),
        }
    }
}

impl FromStr for ServiceCondition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "service_healthy" => Ok(ServiceCondition::ServiceHealthy),
            "service_started" => Ok(ServiceCondition::ServiceStarted),
            _ => Err(Error::invalid_value("depends_on service condition", s)),
        }
    }
}

/// The condition on the container to start first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DependsOnService {
    //TODO: create enum for all supported conditions
    /// A condition indicates that you want a dependency to wait for another container to be “healthy”
    /// (as indicated by a successful state from the healthcheck) before starting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<RawOr<ServiceCondition>>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(DependsOnService, {
    condition, _hidden
});

#[test]
fn service_condition_has_a_string_representation() {
    let pairs = vec![
        (ServiceCondition::ServiceHealthy, "service_healthy"),
        (ServiceCondition::ServiceStarted, "service_started"),
    ];
    for (mode, s) in pairs {
        assert_eq!(mode.to_string(), s);
        assert_eq!(mode, ServiceCondition::from_str(s).unwrap());
    }
}
