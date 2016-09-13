// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A connection from a `Service` to a `Network`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NetworkInterface {
    /// Additional hostnames by which this service will be known on this
    /// network.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<RawOr<String>>,

    // TODO LOW: ipv4_address
    // TODO LOW: ipv6_address

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _phantom: PhantomData<()>,
}

derive_merge_override_for!(NetworkInterface, {
    aliases, _phantom
});
