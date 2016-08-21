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

    /// Docker labels for this container, specifying various sorts of
    /// custom metadata.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_value_list")]
    pub labels: BTreeMap<String, String>,

    /// Links to other services in this file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<AliasedName>,

    // TODO: logging (driver, options)
    // TODO: network_mode
    // TODO: networks (aliases, ipv4_address, ipv6_address)
    // TODO: pid
    // TODO: ports

    /// Security options for AppArmor or SELinux.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_opt: Vec<String>,

    /// The name of the Unix signal which will be sent to stop this
    /// container.  Defaults to SIGTERM if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_signal: Option<String>,

    // TODO: ulimits
    // TODO: volumes_from

    /// The relative number of CPU shares to give to this container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,

    /// Limit the CFS CPU quota.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_quota: Option<u32>,

    // TODO: cpuset

    /// The domain name to use for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domainname: Option<String>,

    /// The hostname to use for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    // TODO: ipc

    /// The MAC address to use for this container's network interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,

    // TODO: mem_limit
    // TODO: memswap_limit

    /// The MAC address to use for this container's network interface.
    #[serde(default, skip_serializing_if = "is_false")]
    pub privileged: bool,

    // TODO: read_only (what is this, anyway?)

    // TODO: restart
    // TODO: shm_size

    /// Should STDIN be left open when running the container?  Corresponds
    /// to `docker run -i`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub stdin_open: bool,

    /// Should a TTY be be allocated for the container?  Corresponds to
    /// `docker run -t`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub tty: bool,

    /// The user name (or UID) of the user under which to execute the
    /// container's command.  May optionally be followed by `:group` or
    /// `:gid` to specific the group or group ID.
    ///
    /// TODO: Parse out optional group field separately?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// The working directory to use for this container.
    ///
    /// TODO: Use PathBuf?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
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
