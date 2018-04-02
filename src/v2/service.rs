// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// A service which will be managed by `docker-compose`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Service {
    /// How to build an image for this service.
    #[serde(default, skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_opt_string_or_struct",
            deserialize_with = "deserialize_opt_string_or_struct")]
    pub build: Option<Build>,

    /// A list of capability names to grant to this container.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_add: Vec<RawOr<String>>,

    /// A list of capability names to revoke from this container.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cap_drop: Vec<RawOr<String>>,

    /// The command-line to run when launching the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<CommandLine>,

    /// The name of an optional parent cgroup.  (Mysterious.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cgroup_parent: Option<RawOr<String>>,

    /// An optional (global, non-scalable) container name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<RawOr<String>>,

    /// A list of devices to map into this container.
    ///
    /// TODO LOW: Add DevicePermissions and make both host and container
    /// mandatory.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub devices: Vec<RawOr<AliasedName>>,

    /// A list of other containers to start first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<RawOr<String>>,

    /// DNS servers.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_item_or_list")]
    pub dns: Vec<RawOr<String>>,

    /// Domains to search for hostnames.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_item_or_list")]
    pub dns_search: Vec<RawOr<String>>,

    /// Locations to mount temporary file systems.
    #[serde(default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_item_or_list")]
    pub tmpfs: Vec<RawOr<String>>,

    /// The entrypoint for the container (wraps `command`, basically).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<CommandLine>,

    /// Environment files used to supply variables to the container.  Note
    /// that this is `env_file` in the underlying Docker format, but the
    /// singular form looks weird at the API level.
    #[serde(rename = "env_file",
            default, skip_serializing_if = "Vec::is_empty",
            deserialize_with = "deserialize_item_or_list")]
    pub env_files: Vec<RawOr<PathBuf>>,

    /// Environment variables and values to supply to the container.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_value_list")]
    pub environment: BTreeMap<String, RawOr<String>>,

    /// Expose a list of ports to any containers that link to us.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expose: Vec<RawOr<String>>,

    /// Extend another service, either in this file or another.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<Extends>,

    /// Links to external containers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_links: Vec<RawOr<AliasedName>>,

    /// Mappings for extra hosts in /etc/hosts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_hosts: Vec<RawOr<HostMapping>>,

    /// Settings for running health checks on services.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<HealthCheck>,

    /// The name of the image to build or pull for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<RawOr<Image>>,

    /// Docker labels for this container, specifying various sorts of
    /// custom metadata.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_key_value_list")]
    pub labels: BTreeMap<String, RawOr<String>>,

    /// Links to other services in this file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<RawOr<AliasedName>>,

    /// Logging options for this container.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,

    /// What networking mode should we use?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_mode: Option<RawOr<NetworkMode>>,

    /// Networks to which this container is attached.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty",
            deserialize_with = "deserialize_map_or_default_list")]
    pub networks: BTreeMap<String, NetworkInterface>,

    /// What PID namespacing mode should we use?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<RawOr<PidMode>>,

    /// What ports do we want to map to our host system?
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<RawOr<PortMapping>>,

    /// Security options for AppArmor or SELinux.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_opt: Vec<RawOr<String>>,

    /// The name of the Unix signal which will be sent to stop this
    /// container.  Defaults to SIGTERM if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_signal: Option<RawOr<String>>,

    // TODO LOW: ulimits

    // TODO LOW: isolation (not documented at this point).

    /// Volumes associated with this service.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<RawOr<VolumeMount>>,

    /// Other places to get volumes from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes_from: Vec<RawOr<VolumesFrom>>,

    /// This will only apply to volumes with no host path and no mapping to
    /// a volume declared under the `volumes` key at the top level of this
    /// file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_driver: Option<RawOr<String>>,

    /// The relative number of CPU shares to give to this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_shares: Option<u32>,

    /// Limit the CFS CPU quota.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_quota: Option<u32>,

    // TODO LOW: cpuset

    /// The domain name to use for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domainname: Option<RawOr<String>>,

    /// The hostname to use for this container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<RawOr<String>>,

    /// What IPC namespacing mode should we use?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipc: Option<RawOr<IpcMode>>,

    /// The MAC address to use for this container's network interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<RawOr<String>>,

    /// The maximum amount of memory which this container may use, in
    /// bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_limit: Option<RawOr<MemorySize>>,

    /// The maximum amount of swap space which this container may use, in
    /// bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memswap_limit: Option<RawOr<MemorySize>>,

    /// The MAC address to use for this container's network interface.
    #[serde(default, skip_serializing_if = "is_false")]
    pub privileged: bool,

    // TODO LOW: read_only (what is this, anyway?)

    /// What should we do when the container exits?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<RawOr<RestartMode>>,

    /// The amount of shared memory to allocate for this container, in
    /// bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shm_size: Option<RawOr<MemorySize>>,

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
    /// TODO LOW: Parse out optional group field separately?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<RawOr<String>>,

    /// The working directory to use for this container.  This is a string,
    /// because on Windows, it will use a different path representation
    /// than the host OS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<RawOr<String>>,

    /// Negative scores make a container less likely to be killed when the
    /// kernel can't find memory; positive scores make it more likely.
    #[serde(skip_serializing_if = "Option::is_none")]
    oom_score_adj: Option<i16>,

    /// Extra groups to grant to the user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    group_add: Vec<String>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_standard_impls_for!(Service, {
    build,
    cap_add,
    cap_drop,
    command,
    cgroup_parent,
    container_name,
    devices,
    depends_on,
    dns,
    dns_search,
    tmpfs,
    entrypoint,
    env_files,
    environment,
    expose,
    extends,
    external_links,
    extra_hosts,
    healthcheck,
    image,
    labels,
    links,
    logging,
    network_mode,
    networks,
    pid,
    ports,
    security_opt,
    stop_signal,
    volumes,
    volumes_from,
    volume_driver,
    cpu_shares,
    cpu_quota,
    domainname,
    hostname,
    ipc,
    mac_address,
    mem_limit,
    memswap_limit,
    privileged,
    restart,
    shm_size,
    stdin_open,
    tty,
    user,
    working_dir,
    oom_score_adj,
    group_add,
    _hidden
});

impl Service {
    /// Inline all our external resources, such as `env_files`, looking up
    /// paths relative to `base`.
    pub fn inline_all(&mut self, base: &Path) -> Result<()> {
        let mut new_env = BTreeMap::new();
        for rel_path in &self.env_files {
            let env_file = EnvFile::load(&base.join(&rel_path.value()?))?;
            new_env.append(&mut env_file.to_environment()?);
        }
        new_env.append(&mut self.environment.clone());
        self.environment = new_env;
        self.env_files.clear();
        Ok(())
    }
}

#[test]
fn service_handles_sample_fields_correctly() {
    let yaml = r#"---
dns:
  - 8.8.8.8
dns_search:
  - example.com
  - example.net
image: hello
"#;
    assert_roundtrip!(Service, yaml);
}

#[test]
fn service_env_file_is_renamed() {
    let yaml = r#"---
env_file:
  - foo/bar.env
"#;
    let service: Service = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(service.env_files.len(), 1);
    assert_eq!(service.env_files[0], escape("foo/bar.env").unwrap());
}

#[test]
fn service_networks_supports_map() {
    let yaml = r#"---
networks:
  backend:
    aliases:
      - hostname2
  frontend: {}
"#;
    assert_roundtrip!(Service, yaml);
}

#[test]
fn service_networks_supports_list() {
    let yaml = r#"---
"networks":
  - "backend"
"#;
    let service: Service = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(service.networks.len(), 1);
    assert_eq!(service.networks.get("backend").unwrap(),
               &NetworkInterface::default());
}
