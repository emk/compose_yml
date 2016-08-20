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

    // devices

    /// A list of other containers to start first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,

    // dns
    // dns_search
    // tmpfs

    /// The entrypoint for the container (wraps `command`, basically).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<CommandLine>,

    // env_file
    // environment

    /// Expose a list of ports to any containers that link to us.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expose: Vec<String>,

    // extends
    // external_links
    // extra_hosts

    /// The name of the image to build or pull for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    // labels
    // links
    // logging (driver, options)
    // network_mode
    // networks (aliases, ipv4_address, ipv6_address
    // pid
    // ports
    // security_opt
    // stop_signal
    // ulimits
    // volumes_from
    // cpu_shares, cpu_quota, cpuset, domainname, hostname, ipc, mac_address, mem_limit, memswap_limit, privileged, read_only, restart, shm_size, stdin_open, tty, user, working_dir
}
