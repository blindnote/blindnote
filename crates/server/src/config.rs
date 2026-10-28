//! Runtime configuration, read from the process environment.
//!
//! There is no configuration file. The binary is a single artifact whose
//! behaviour is described entirely by its environment, so that the running
//! configuration is visible in the unit that starts the container rather than
//! in a file on the host that nobody diffs.
//!
//! Parsing is factored into free functions that take a value rather than read
//! one, because the tests cannot set an environment variable: `std::env::set_var`
//! is `unsafe` in edition 2024 and this crate forbids `unsafe`. Keeping the
//! parsing pure is what keeps it testable at all.

use std::env::{self, VarError};
use std::fmt::Display;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

/// Prefix shared by every environment variable this binary reads.
const ENV_PREFIX: &str = "BLINDNOTE_";

/// Everything the server needs in order to start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Address the HTTP listener binds to.
    ///
    /// Loopback by default, on purpose: the only thing that should ever reach
    /// this port is the TLS terminator running on the same host.
    pub bind: SocketAddr,

    /// Directory holding the two SQLite files.
    ///
    /// In production this is a host bind mount, never a Docker named volume --
    /// `docker compose down -v` deletes named volumes, and a single wrong flag
    /// must not be able to destroy every pending note.
    pub data_dir: PathBuf,
}

impl Config {
    /// Reads the configuration from the environment, applying defaults.
    ///
    /// An empty value counts as unset, so that a compose file can leave a
    /// variable in place without having to delete the line to get the default.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a variable is set but cannot be read as
    /// UTF-8, or cannot be parsed into the type the field expects. A variable
    /// that is simply absent is not an error; its default applies.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            bind: from_env_parsed(
                "BIND",
                "socket address",
                SocketAddr::from((Ipv4Addr::LOCALHOST, 8080)),
            )?,
            data_dir: from_env_raw("DATA_DIR")?
                .map_or_else(|| PathBuf::from("data"), PathBuf::from),
        })
    }
}

/// Why a configuration could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The variable is set, but its value is not valid UTF-8.
    #[error("{key} is not valid UTF-8")]
    NotUnicode {
        /// Full name of the offending variable.
        key: String,
    },

    /// The variable is set, but could not be parsed into the expected type.
    ///
    /// The value itself is deliberately absent from the message. This error is
    /// logged, and configuration is the kind of place a secret ends up.
    #[error("{key} is not a valid {expected}: {reason}")]
    Invalid {
        /// Full name of the offending variable.
        key: String,
        /// Human-readable name of the type that was expected.
        expected: &'static str,
        /// What the parser said, without the value it was given.
        reason: String,
    },
}

/// Reads `BLINDNOTE_<name>`, treating an empty value as unset.
fn from_env_raw(name: &str) -> Result<Option<String>, ConfigError> {
    let key = format!("{ENV_PREFIX}{name}");
    match env::var(&key) {
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode { key }),
    }
}

/// Reads `BLINDNOTE_<name>` and parses it, falling back to `default` when unset.
fn from_env_parsed<T>(name: &str, expected: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    match from_env_raw(name)? {
        None => Ok(default),
        Some(value) => parse(name, expected, &value),
    }
}

/// Parses one already-read value, naming the variable in any error.
fn parse<T>(name: &str, expected: &'static str, value: &str) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    value
        .parse()
        .map_err(|reason: T::Err| ConfigError::Invalid {
            key: format!("{ENV_PREFIX}{name}"),
            expected,
            reason: reason.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, parse};
    use std::net::SocketAddr;

    #[test]
    fn parse_accepts_a_socket_address() {
        let parsed: SocketAddr =
            parse("BIND", "socket address", "0.0.0.0:8080").expect("valid address");

        assert_eq!(parsed, SocketAddr::from(([0, 0, 0, 0], 8080)));
    }

    #[test]
    fn parse_names_the_variable_it_rejected() {
        let error =
            parse::<SocketAddr>("BIND", "socket address", "not-an-address").expect_err("rejected");

        let ConfigError::Invalid { key, expected, .. } = &error else {
            panic!("expected ConfigError::Invalid, got {error:?}");
        };
        assert_eq!(key, "BLINDNOTE_BIND");
        assert_eq!(*expected, "socket address");
    }

    #[test]
    fn parse_errors_do_not_quote_the_rejected_value() {
        let error = parse::<SocketAddr>("BIND", "socket address", "super-secret-value")
            .expect_err("rejected");

        assert!(
            !error.to_string().contains("super-secret-value"),
            "a configuration error must not echo the value it was given: {error}"
        );
    }
}
