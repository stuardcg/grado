//! The configuration every runtime starts from (ADR 0011): `grado.toml` in the working directory,
//! if it's there, then every `GRADO_*` environment variable on top. `grado.example.toml` documents
//! each setting.

use std::fmt;
use std::net::SocketAddr;
use std::path::Path;

use application::AgentToken;
use domain::stamp::StampSetting;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::value::{Dict, Value};
use figment::{Figment, Provider};

/// The file read from the working directory, when it's there.
pub const FILE: &str = "grado.toml";
const ENV_PREFIX: &str = "GRADO_";
/// Joins a section and its key in an environment variable name.
const ENV_NESTING: &str = "__";

/// Every setting, parsed and checked.
pub struct Config {
    /// `addr`: the address the local server listens on.
    pub addr: SocketAddr,
    pub dev: DevConfig,
}

/// `dev`: the sample data's settings. Dev only; they go away with STU-27 and STU-29.
pub struct DevConfig {
    /// `dev.agent_token`: the token of the sample player's agent connection.
    pub agent_token: AgentToken,
    /// `dev.stamp`: whether a step waits for a stamp after approval.
    pub stamp: StampSetting,
}

impl Config {
    /// Loads `grado.toml`, if it's in the working directory, and the `GRADO_*` environment
    /// variables, which win over the file.
    pub fn from_env() -> Result<Self, ConfigError> {
        let file = Path::new(FILE);
        let mut figment = Figment::new();
        if file.exists() {
            figment = figment.merge(Toml::file_exact(file));
        }
        // Each value stays the text it was set to: figment's own `Env` would read `007` as 7.
        let figment = Env::prefixed(ENV_PREFIX)
            .split(ENV_NESTING)
            .iter()
            .fold(figment, |figment, (key, text)| figment.merge(Serialized::default(key.as_str(), text)));
        Self::load(&figment)
    }

    /// Parses every setting out of `figment`, or lists every problem with it.
    pub fn load(figment: &Figment) -> Result<Self, ConfigError> {
        let mut settings = Reader::new(figment)?;
        let addr =
            settings.read("addr", "a socket address, such as 127.0.0.1:8080", |text| text.parse().ok());
        let agent_token = settings.read("dev.agent_token", "a token that isn't blank", AgentToken::new);
        let stamp = settings.read("dev.stamp", "on or off", stamp);
        settings.finish()?;

        let (Some(addr), Some(agent_token), Some(stamp)) = (addr, agent_token, stamp) else {
            unreachable!("a setting that wasn't read is a problem, so `finish` returned it");
        };
        Ok(Self { addr, dev: DevConfig { agent_token, stamp } })
    }
}

fn stamp(text: String) -> Option<StampSetting> {
    match text.as_str() {
        "on" => Some(StampSetting::On),
        "off" => Some(StampSetting::Off),
        _ => None,
    }
}

/// Reads one setting at a time, so a bad one doesn't hide the next.
struct Reader<'a> {
    figment: &'a Figment,
    given: Vec<Dict>,
    read: Vec<&'static str>,
    problems: Vec<Problem>,
}

impl<'a> Reader<'a> {
    fn new(figment: &'a Figment) -> Result<Self, ConfigError> {
        let given = figment
            .data()
            .map_err(|error| ConfigError { problems: vec![Problem::Unreadable(error.to_string())] })?
            .into_values()
            .collect();
        Ok(Self { figment, given, read: Vec::new(), problems: Vec::new() })
    }

    /// The setting at `key`, if it's text that `parse` accepts; otherwise it records a problem.
    fn read<T>(
        &mut self,
        key: &'static str,
        expected: &'static str,
        parse: impl FnOnce(String) -> Option<T>,
    ) -> Option<T> {
        self.read.push(key);
        let problem = match self.figment.extract_inner::<String>(key) {
            Ok(text) => match parse(text) {
                Some(value) => return Some(value),
                None => Problem::Invalid { key, expected },
            },
            Err(error) if error.missing() => Problem::Missing(key),
            Err(_) => Problem::NotText(key),
        };
        self.problems.push(problem);
        None
    }

    /// Every problem the reads found, then every given key that isn't a setting.
    fn finish(mut self) -> Result<(), ConfigError> {
        for dict in &self.given {
            unknown(dict, "", &self.read, &mut self.problems);
        }
        if self.problems.is_empty() { Ok(()) } else { Err(ConfigError { problems: self.problems }) }
    }
}

/// Records each key under `prefix` in `dict` that no read asked for.
fn unknown(dict: &Dict, prefix: &str, read: &[&str], problems: &mut Vec<Problem>) {
    for (name, value) in dict {
        let key = if prefix.is_empty() { name.clone() } else { format!("{prefix}.{name}") };
        if read.contains(&key.as_str()) {
            continue;
        }
        match value {
            Value::Dict(_, inner) => unknown(inner, &key, read, problems),
            _ => problems.push(Problem::Unknown(key)),
        }
    }
}

/// Why the server can't start: every problem with the configuration.
#[derive(Debug)]
pub struct ConfigError {
    pub problems: Vec<Problem>,
}

/// One thing wrong with the configuration. It never holds a setting's value.
#[derive(Debug, PartialEq, Eq)]
pub enum Problem {
    /// `grado.toml` couldn't be read or parsed; figment's message.
    Unreadable(String),
    /// A setting nothing gave.
    Missing(&'static str),
    /// A setting given as something other than text, such as a TOML number.
    NotText(&'static str),
    /// A setting whose text isn't what it takes.
    Invalid { key: &'static str, expected: &'static str },
    /// A key that isn't a setting, often a typo.
    Unknown(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.problems.len();
        write!(f, "the configuration has {count} problem{}:", if count == 1 { "" } else { "s" })?;
        for problem in &self.problems {
            write!(f, "\n- {problem}")?;
        }
        write!(f, "\nEvery setting is listed in grado.example.toml.")
    }
}

impl std::error::Error for ConfigError {}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(message) => write!(f, "{FILE} can't be read: {message}"),
            Self::Missing(key) => write!(f, "{}: missing", Named(key)),
            Self::NotText(key) => write!(f, "{}: not text; quote it in {FILE}", Named(key)),
            Self::Invalid { key, expected } => write!(f, "{}: not {expected}", Named(key)),
            Self::Unknown(key) => write!(f, "{}: not a Grado setting", Named(key)),
        }
    }
}

/// A key with its environment variable name, such as `dev.stamp (GRADO_DEV__STAMP)`.
struct Named<'a>(&'a str);

impl fmt::Display for Named<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let env = self.0.replace('.', ENV_NESTING).to_ascii_uppercase();
        write!(f, "{} ({ENV_PREFIX}{env})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use figment::Jail;

    use super::*;

    const EXAMPLE: &str = include_str!("../../../grado.example.toml");

    /// Runs `test` in an empty directory with no environment variables.
    #[expect(clippy::result_large_err, reason = "`Jail::expect_with` fixes the closure's error type")]
    fn jail(test: impl FnOnce(&mut Jail)) {
        Jail::expect_with(|jail| {
            jail.clear_env();
            test(jail);
            Ok(())
        });
    }

    fn problems(result: Result<Config, ConfigError>) -> Vec<Problem> {
        match result {
            Ok(_) => panic!("the configuration loaded"),
            Err(error) => error.problems,
        }
    }

    #[test]
    fn the_example_file_is_a_complete_configuration() {
        jail(|jail| {
            jail.create_file(FILE, EXAMPLE).unwrap();
            let config = Config::from_env().expect("the example loads");
            assert_eq!(config.addr, "127.0.0.1:8080".parse().unwrap());
            assert_eq!(config.dev.agent_token, AgentToken::new("dev-token").unwrap());
            assert_eq!(config.dev.stamp, StampSetting::Off);
        });
    }

    #[test]
    fn the_environment_alone_is_enough() {
        jail(|jail| {
            jail.set_env("GRADO_ADDR", "127.0.0.1:9000");
            jail.set_env("GRADO_DEV__AGENT_TOKEN", "token");
            jail.set_env("GRADO_DEV__STAMP", "on");
            let config = Config::from_env().expect("the environment loads");
            assert_eq!(config.addr, "127.0.0.1:9000".parse().unwrap());
            assert_eq!(config.dev.stamp, StampSetting::On);
        });
    }

    #[test]
    fn the_environment_wins_and_keeps_its_text() {
        jail(|jail| {
            jail.create_file(FILE, EXAMPLE).unwrap();
            jail.set_env("GRADO_DEV__AGENT_TOKEN", "007");
            let config = Config::from_env().expect("the configuration loads");
            assert_eq!(config.dev.agent_token, AgentToken::new("007").unwrap());
        });
    }

    #[test]
    fn every_problem_is_listed_at_once() {
        jail(|jail| {
            jail.create_file(FILE, "addr = \"localhost\"\n[dev]\nstamp = \"yes\"\n").unwrap();
            jail.set_env("GRADO_DEV__STMP", "on");
            assert_eq!(
                problems(Config::from_env()),
                [
                    Problem::Invalid { key: "addr", expected: "a socket address, such as 127.0.0.1:8080" },
                    Problem::Missing("dev.agent_token"),
                    Problem::Invalid { key: "dev.stamp", expected: "on or off" },
                    Problem::Unknown("dev.stmp".into()),
                ]
            );
        });
    }

    #[test]
    fn a_blank_token_is_refused() {
        jail(|jail| {
            jail.create_file(FILE, EXAMPLE).unwrap();
            jail.set_env("GRADO_DEV__AGENT_TOKEN", " ");
            assert_eq!(
                problems(Config::from_env()),
                [Problem::Invalid { key: "dev.agent_token", expected: "a token that isn't blank" }]
            );
        });
    }

    #[test]
    fn a_number_in_the_file_isnt_text() {
        let figment = Figment::from(Toml::string(&EXAMPLE.replace("\"127.0.0.1:8080\"", "8080")));
        assert_eq!(problems(Config::load(&figment)), [Problem::NotText("addr")]);
    }

    #[test]
    fn a_file_that_doesnt_parse_is_unreadable() {
        let figment = Figment::from(Toml::string("addr = ["));
        assert!(matches!(problems(Config::load(&figment))[..], [Problem::Unreadable(_)]));
    }

    #[test]
    fn a_problem_names_the_key_and_its_variable_but_not_the_value() {
        let error =
            ConfigError { problems: vec![Problem::Invalid { key: "dev.stamp", expected: "on or off" }] };
        assert_eq!(
            error.to_string(),
            "the configuration has 1 problem:\n\
             - dev.stamp (GRADO_DEV__STAMP): not on or off\n\
             Every setting is listed in grado.example.toml."
        );
    }
}
