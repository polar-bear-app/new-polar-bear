use anyhow::Result;
use serde::Deserialize;
use std::{
    borrow::Cow,
    collections::BTreeMap,
    env::VarError,
    ops::Deref,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub build: Option<Build>,
    /// <https://doc.rust-lang.org/cargo/reference/config.html#env>
    pub env: Option<BTreeMap<String, EnvOption>>,
}

impl Config {
    pub fn parse_from_toml(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Debug)]
pub struct LocalizedConfig {
    pub config: Config,
    /// The directory containing `./.cargo/config.toml`
    pub workspace: PathBuf,
}

impl Deref for LocalizedConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl LocalizedConfig {
    pub fn new(workspace: PathBuf) -> Result<Self> {
        Ok(Self {
            config: Config::parse_from_toml(workspace.join(".cargo/config.toml"))?,
            workspace,
        })
    }

    /// Search for and open `.cargo/config.toml` in any parent of the workspace root path.
    // TODO: Find, parse, and merge _all_ config files following the hierarchical structure:
    // https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure
    pub fn find_cargo_config_for_workspace(workspace: impl AsRef<Path>) -> Result<Option<Self>> {
        let workspace = workspace.as_ref();
        let workspace = dunce::canonicalize(workspace)?;
        workspace
            .ancestors()
            .find(|dir| dir.join(".cargo/config.toml").is_file())
            .map(|p| p.to_path_buf())
            .map(Self::new)
            .transpose()
    }

    /// Propagate environment variables from this `.cargo/config.toml` to the process environment
    /// using [`std::env::set_var()`].
    ///
    /// Note that this is automatically performed when calling [`Subcommand::new()`][super::Subcommand::new()].
    pub fn set_env_vars(&self) -> Result<()> {
        if let Some(env) = &self.config.env {
            for (key, env_option) in env {
                // Existing environment variables always have precedence unless
                // the extended format is used to set `force = true`:
                if !matches!(env_option, EnvOption::Value { force: true, .. })
                    && std::env::var_os(key).is_some()
                {
                    continue;
                }

                std::env::set_var(key, env_option.resolve_value(&self.workspace)?.as_ref())
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Build {
    pub target_dir: Option<String>,
}

/// Serializable environment variable in cargo config, configurable as per
/// <https://doc.rust-lang.org/cargo/reference/config.html#env>,
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EnvOption {
    String(String),
    Value {
        value: String,
        #[serde(default)]
        force: bool,
        #[serde(default)]
        relative: bool,
    },
}

impl EnvOption {
    /// Retrieve the value and join it to `config_parent` when [`EnvOption::Value::relative`] is set.
    ///
    /// `config_parent` is the directory containing `.cargo/config.toml` where this was parsed from.
    pub fn resolve_value(&self, config_parent: impl AsRef<Path>) -> Result<Cow<'_, str>> {
        Ok(match self {
            Self::Value {
                value,
                relative: true,
                force: _,
            } => config_parent
                .as_ref()
                .join(value)
                .into_os_string()
                .into_string()
                .map_err(VarError::NotUnicode)?
                .into(),
            Self::String(value) | Self::Value { value, .. } => value.into(),
        })
    }
}

#[test]
fn test_env_parsing() {
    let toml = r#"
[env]
# Set ENV_VAR_NAME=value for any process run by Cargo
ENV_VAR_NAME = "value"
# Set even if already present in environment
ENV_VAR_NAME_2 = { value = "value", force = true }
# Value is relative to .cargo directory containing `config.toml`, make absolute
ENV_VAR_NAME_3 = { value = "relative/path", relative = true }"#;

    let mut env = BTreeMap::new();
    env.insert(
        "ENV_VAR_NAME".to_string(),
        EnvOption::String("value".into()),
    );
    env.insert(
        "ENV_VAR_NAME_2".to_string(),
        EnvOption::Value {
            value: "value".into(),
            force: true,
            relative: false,
        },
    );
    env.insert(
        "ENV_VAR_NAME_3".to_string(),
        EnvOption::Value {
            value: "relative/path".into(),
            force: false,
            relative: true,
        },
    );

    assert_eq!(
        toml::from_str::<Config>(toml),
        Ok(Config {
            build: None,
            env: Some(env)
        })
    );
}

#[test]
fn test_env_precedence_rules() {
    let toml = r#"
[env]
CARGO_SUBCOMMAND_TEST_ENV_NOT_FORCED = "not forced"
CARGO_SUBCOMMAND_TEST_ENV_FORCED = { value = "forced", force = true }"#;

    let config = LocalizedConfig {
        config: toml::from_str::<Config>(toml).unwrap(),
        workspace: PathBuf::new(),
    };

    // Check if all values are propagated to the environment
    config.set_env_vars().unwrap();

    assert!(matches!(
        std::env::var("CARGO_SUBCOMMAND_TEST_ENV_NOT_SET"),
        Err(VarError::NotPresent)
    ));
    assert_eq!(
        std::env::var("CARGO_SUBCOMMAND_TEST_ENV_NOT_FORCED").unwrap(),
        "not forced"
    );
    assert_eq!(
        std::env::var("CARGO_SUBCOMMAND_TEST_ENV_FORCED").unwrap(),
        "forced"
    );

    // Set some environment values
    std::env::set_var(
        "CARGO_SUBCOMMAND_TEST_ENV_NOT_FORCED",
        "not forced process environment value",
    );
    std::env::set_var(
        "CARGO_SUBCOMMAND_TEST_ENV_FORCED",
        "forced process environment value",
    );

    config.set_env_vars().unwrap();

    assert_eq!(
        std::env::var("CARGO_SUBCOMMAND_TEST_ENV_NOT_FORCED").unwrap(),
        // Value remains what is set in the process environment,
        // and is not overwritten by set_env_vars()
        "not forced process environment value"
    );
    assert_eq!(
        std::env::var("CARGO_SUBCOMMAND_TEST_ENV_FORCED").unwrap(),
        // Value is overwritten thanks to force=true, despite
        // also being set in the process environment
        "forced"
    );
}

#[test]
fn test_env_relative() {
    let toml = r#"
[env]
CARGO_SUBCOMMAND_TEST_ENV_SRC_DIR = { value = "src", force = true, relative = true }
"#;

    let config = LocalizedConfig {
        config: toml::from_str::<Config>(toml).unwrap(),
        // Path does not have to exist
        workspace: PathBuf::from("my/work/space"),
    };

    config.set_env_vars().unwrap();

    let path = std::env::var("CARGO_SUBCOMMAND_TEST_ENV_SRC_DIR")
        .expect("Relative env var should always be set");
    let path = PathBuf::from(path);
    assert!(
        path.is_relative(),
        "Workspace was not absolute so this shouldn't either"
    );
    assert_eq!(path, Path::new("my/work/space/src"));
}
