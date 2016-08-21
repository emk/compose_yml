// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A service which will be managed by `docker-compose`.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    /// How to build an image for this service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<Build>,

    /// A list of capability names to grant to this container.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_add: Vec<String>,

    /// A list of capability names to revoke from this container.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_drop: Vec<String>,

    /// The command-line to run when launching the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<CommandLine>,

    /// The name of an optional parent cgroup.  (Mysterious.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cgroup_parent: Option<String>,

    /// An optional (global, non-scalable) container name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,

    /// A list of devices to map into this container.
    ///
    /// TODO: Permissions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<AliasedName>,

    /// A list of other containers to start first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,

    /// DNS servers.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            serialize_with = "serialize_item_or_list",
            deserialize_with = "deserialize_string_or_list")]
    pub dns: Vec<String>,

    /// Domains to search for hostnames.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            serialize_with = "serialize_item_or_list",
            deserialize_with = "deserialize_string_or_list")]
    pub dns_search: Vec<String>,

    /// Locations to mount temporary file systems.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            serialize_with = "serialize_item_or_list",
            deserialize_with = "deserialize_string_or_list")]
    pub tmpfs: Vec<String>,

    /// The entrypoint for the container (wraps `command`, basically).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<CommandLine>,

    /// Environment files used to supply variables to the container.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            serialize_with = "serialize_item_or_list",
            deserialize_with = "deserialize_string_or_list")]
    pub env_file: Vec<String>,

    /// Environment variables and values to supply to the container.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_value_list")]
    pub environment: BTreeMap<String, String>,

    /// Expose a list of ports to any containers that link to us.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expose: Vec<String>,

    // TODO: extends

    /// Links to external containers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_links: Vec<AliasedName>,

    // TODO: extra_hosts

    /// The name of the image to build or pull for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    // TODO: labels

    /// Links to other services in this file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<AliasedName>,

    // TODO: logging (driver, options)
    // TODO: network_mode
    // TODO: networks (aliases, ipv4_address, ipv6_address
    // TODO: pid
    // TODO: ports
    // TODO: security_opt
    // TODO: stop_signal
    // TODO: ulimits
    // TODO: volumes_from
    // TODO: cpu_shares, cpu_quota, cpuset, domainname, hostname, ipc, mac_address, mem_limit, memswap_limit, privileged, read_only, restart, shm_size, stdin_open, tty, user, working_dir
}

#[test]
fn service_handles_sample_fields_correctly() {
    let yaml = r#"---
"dns": "8.8.8.8"
"dns_search":
  - "example.com"
  - "example.net"
"image": "hello"
"#;
    assert_roundtrip!(Service, yaml);
}
