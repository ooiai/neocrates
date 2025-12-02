use serde::Deserialize;
use serde_yaml;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn load_config_from_file<T, P>(path: P) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<std::path::Path>,
{
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    serde_yaml::from_str(&contents).ok()
}

// Loads configuration from a specific YAML file path.
///
/// This function is similar to `load_config_from_file` but more explicitly accepts
/// a path parameter and provides better documentation. It is useful when you need
/// to load configuration from non-standard locations or specifically named files.
///
/// # Arguments
/// * `config_path` - The path to the configuration file to load.
///
/// # Returns
/// `Some(T)` if the configuration file is found and parsed successfully,
/// `None` otherwise.
///
/// # Example
/// ```
/// use serde::Deserialize;
/// use std::path::Path;
///
/// #[derive(Deserialize)]
/// struct AppConfig {
///     server_port: u16,
///     debug_mode: bool,
/// }
///
/// if let Some(config) = loader::load_named_config::<AppConfig>(Path::new("custom-config.yml")) {
///     println!("Server port: {}", config.server_port);
/// } else {
///     eprintln!("Failed to load configuration");
/// }
/// ```
pub fn load_named_config<T>(config_path: &Path) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    load_config_from_file(config_path)
}

/// Loads configuration from environment-specific or default YAML files.
///
/// This function searches for configuration files in the following order:
/// 1. `application.{ENV}.yml`
/// 2. `application.{ENV}.yaml`
/// 3. `application.yml`
/// 4. `application.yaml`
///
/// Where `ENV` is the value of the environment variable "ENV". If "ENV" is not set,
/// it defaults to searching `config.yml` and `config.yaml`.
///
/// # Returns
/// `Some(T)` if a valid configuration file is found and parsed successfully,
/// `None` otherwise.
///
/// # Example
/// ```
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct AppConfig {
///     server_port: u16,
///     debug_mode: bool,
/// }
///
/// // Set ENV variable (optional)
/// std::env::set_var("ENV", "production");
///
/// if let Some(config) = loader::load_config::<AppConfig>() {
///     println!("Server port: {}", config.server_port);
/// } else {
///     eprintln!("Failed to load configuration");
/// }
/// ```
pub fn load_config<T>() -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let env_var = env::var("ENV").ok();

    let candidates = match env_var.as_deref() {
        Some(env) if !env.is_empty() => {
            vec![
                format!("application.{}.yml", env),
                format!("application.{}.yaml", env),
            ]
        }
        _ => vec![
            "application.yml".to_string(),
            "application.yaml".to_string(),
        ],
    };

    for file_name in candidates {
        if let Some(config) = load_config_from_file::<T, _>(&file_name) {
            return Some(config);
        }
    }

    None
}
