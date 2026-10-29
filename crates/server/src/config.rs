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
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

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

    /// Address of the reverse proxy whose `X-Forwarded-For` may be believed.
    ///
    /// Unset by default, and unset means "believe nobody": every request is
    /// attributed to the address it actually arrived from. That is the safe
    /// direction. The unsafe direction -- trusting a header any client can
    /// send -- gives you a rate limiter that is bypassed by setting it.
    pub trusted_proxy: Option<IpAddr>,

    /// Sustained requests allowed per client address, per second.
    pub requests_per_second: u64,

    /// How far a single client may run ahead of its sustained rate.
    pub request_burst: u32,

    /// Notes one address may create per hour, on top of the rate limit.
    ///
    /// The rate limit stops a flood; this stops a slow drip that would fill
    /// the disk over a day. Both are needed, and they are different numbers.
    pub creates_per_hour: u32,

    /// Stop accepting new notes when the data volume has less than this free.
    ///
    /// A full disk is the one ordinary failure that can destroy a note
    /// somebody is waiting to read: SQLite cannot commit, the log cannot grow
    /// and the sweeper cannot delete. So creation is refused while there is
    /// still room to finish serving what is already stored. Reading and
    /// reporting stay open, because both make room.
    ///
    /// Zero disables the check.
    pub min_free_bytes: u64,

    /// How often expired notes and their keys are deleted.
    pub sweep_interval: Duration,

    /// How long one request may take before it is cut off.
    pub request_timeout: Duration,

    /// How many requests may be in flight at once.
    ///
    /// Every note operation is a write that waits for an `fsync`, so the
    /// useful ceiling is set by the disk rather than by the CPU. This bound
    /// exists so that a flood queues rather than allocating without limit.
    pub max_concurrent_requests: usize,
}

impl Default for Config {
    /// The configuration of a server started with an empty environment.
    ///
    /// Every default lives here and nowhere else: [`Config::from_env`] reads
    /// these values and overrides the ones the environment names. Two lists of
    /// defaults would eventually disagree, and the one that matters would be
    /// whichever the reader did not open.
    fn default() -> Self {
        Self {
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 8080)),
            data_dir: PathBuf::from("data"),
            trusted_proxy: None,
            requests_per_second: 5,
            request_burst: 20,
            creates_per_hour: 60,
            // 256 MiB. Large enough that the write-ahead logs, a sweep and
            // every note already stored can all finish; small enough not to
            // waste a volume. Roughly four thousand full-size notes of
            // headroom.
            min_free_bytes: 256 * 1024 * 1024,
            sweep_interval: Duration::from_secs(60),
            request_timeout: Duration::from_secs(15),
            max_concurrent_requests: 256,
        }
    }
}

impl Config {
    /// Reads the configuration from the environment, over the defaults.
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
        let defaults = Self::default();

        Ok(Self {
            bind: from_env_parsed("BIND", "socket address", defaults.bind)?,
            data_dir: from_env_raw("DATA_DIR")?.map_or(defaults.data_dir, PathBuf::from),
            trusted_proxy: match from_env_raw("TRUSTED_PROXY")? {
                None => defaults.trusted_proxy,
                Some(value) => Some(parse("TRUSTED_PROXY", "IP address", &value)?),
            },
            requests_per_second: from_env_parsed(
                "REQUESTS_PER_SECOND",
                "positive integer",
                defaults.requests_per_second,
            )?,
            request_burst: from_env_parsed(
                "REQUEST_BURST",
                "positive integer",
                defaults.request_burst,
            )?,
            creates_per_hour: from_env_parsed(
                "CREATES_PER_HOUR",
                "positive integer",
                defaults.creates_per_hour,
            )?,
            min_free_bytes: from_env_parsed(
                "MIN_FREE_BYTES",
                "positive integer",
                defaults.min_free_bytes,
            )?,
            sweep_interval: from_env_seconds("SWEEP_INTERVAL_SECONDS", defaults.sweep_interval)?,
            request_timeout: from_env_seconds("REQUEST_TIMEOUT_SECONDS", defaults.request_timeout)?,
            max_concurrent_requests: from_env_parsed(
                "MAX_CONCURRENT_REQUESTS",
                "positive integer",
                defaults.max_concurrent_requests,
            )?,
        })
    }
}

/// Reads `BLINDNOTE_<name>` as a number of whole seconds.
fn from_env_seconds(name: &str, default: Duration) -> Result<Duration, ConfigError> {
    match from_env_raw(name)? {
        None => Ok(default),
        Some(value) => Ok(Duration::from_secs(parse(
            name,
            "positive integer",
            &value,
        )?)),
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
