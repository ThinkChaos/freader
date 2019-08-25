use std::env::VarError;
use std::fmt::{Debug, Display};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub http_host: String,
    pub http_port: u16,

    pub auth_username: String,
    pub auth_password: String,

    pub sqlite_db: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Config {
            http_host: Self::var("HOST")?,
            http_port: Self::var("PORT")?,

            auth_username: Self::var("USERNAME")?,
            auth_password: Self::var("PASSWORD")?,

            sqlite_db: Self::var_or("SQLITE_DB", "file:ggrrss.sqlite")?,
        })
    }

    fn var<K, V>(key: K) -> Result<V, Error>
    where
        K: Display,
        V: FromStr,
        V::Err: Display,
    {
        let key = format!("GGRRSS_{}", key);
        match std::env::var(&key) {
            Ok(val) => val
                .parse()
                .map_err(|err| Error::ValueConversion(key, format!("{}", err))),
            Err(VarError::NotPresent) => Err(Error::Missing(key)),
            Err(VarError::NotUnicode(_)) => Err(Error::ValueNotUnicode(key)),
        }
    }

    fn var_or<K, V, D>(key: K, default: D) -> Result<V, Error>
    where
        K: Display,
        V: Debug + FromStr,
        V::Err: Display,
        D: Into<V>,
    {
        match Self::var(key) {
            Err(Error::Missing(key)) => {
                let val: V = default.into();
                log::debug!("{} is not defined, using default: {:?}", key, val);
                Ok(val)
            }
            res => res,
        }
    }
}


pub enum Error {
    Missing(String),
    ValueNotUnicode(String),
    ValueConversion(String, String),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Missing(key) => write!(fmt, "Missing value for {}", key),
            Self::ValueNotUnicode(key) => {
                write!(fmt, "Could not parse {}'s value as a UTF-8 string", key)
            }
            Self::ValueConversion(key, err_msg) => {
                write!(fmt, "Could not parse {}'s value: {}", key, err_msg)
            }
        }
    }
}
