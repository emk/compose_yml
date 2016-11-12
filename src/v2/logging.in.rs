// This is not a normal Rust module! It's included directly into v2.rs,
// possibly after build-time preprocessing.  See v2.rs for an explanation
// of how this works.

/// Logging configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Logging {
    /// The logging driver to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<RawOr<String>>,

    /// Options to pass to the log driver.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, RawOr<String>>,

    /// PRIVATE.  Mark this struct as having unknown fields for future
    /// compatibility.  This prevents direct construction and exhaustive
    /// matching.  This needs to be be public because of
    /// http://stackoverflow.com/q/39277157/12089
    #[doc(hidden)]
    #[serde(default, skip_serializing, skip_deserializing)]
    pub _hidden: (),
}

derive_interpolate_all_for!(Logging, {
    driver, options, _hidden
});

impl MergeOverride for Logging {
    fn merge_override(&self, ovr: &Self) -> Self {
        Logging {
            driver: self.driver.merge_override(&ovr.driver),
            // Don't merge options if driver has changed.
            options: if &self.driver == &ovr.driver || ovr.driver.is_none() {
                self.options.merge_override(&ovr.options)
            } else {
                ovr.options.clone()
            },
            _hidden: (),
        }
    }
}

#[test]
fn logging_merges_options_on_merge_if_driver_stays_the_same() {
    let yaml1 = r#"---
driver: "d1"
options:
  opt1: "value1"
"#;
    let logging1: Logging = serde_yaml::from_str(yaml1).unwrap();
    let yaml2 = r#"---
driver: "d1"
options:
  opt2: "value2"
"#;
    let logging2: Logging = serde_yaml::from_str(yaml2).unwrap();

    let merged = logging1.merge_override(&logging2);
    assert_eq!(merged.driver.unwrap().value().unwrap(), "d1");
    assert_eq!(merged.options.get("opt1").expect("should have opt1").value().unwrap(),
               "value1");
    assert_eq!(merged.options.get("opt2").expect("should have opt2").value().unwrap(),
               "value2");
}

#[test]
fn logging_clears_options_on_merge_if_driver_changed() {
    let yaml1 = r#"---
driver: "d1"
options:
  opt1: "value1"
"#;
    let logging1: Logging = serde_yaml::from_str(yaml1).unwrap();
    let yaml2 = r#"---
driver: "d2"
options:
  opt2: "value2"
"#;
    let logging2: Logging = serde_yaml::from_str(yaml2).unwrap();

    let merged = logging1.merge_override(&logging2);
    assert_eq!(merged.driver.unwrap().value().unwrap(), "d2");
    assert!(merged.options.get("opt1").is_none());
    assert_eq!(merged.options.get("opt2").expect("should have opt2").value().unwrap(),
               "value2");
}
