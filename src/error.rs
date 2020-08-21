use core::result;
use std::error::Error as StdError;
use std::fmt;
use std::io;

use num_bigint::BigInt;
use serde::de;
use serde::ser;

#[derive(Debug)]
pub enum Unexpected {
    Bool(bool),
    Integer(BigInt),
    Float(f64),
    Str(String),
    Unit,
    Array,
    Map,
}

impl fmt::Display for Unexpected {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Unexpected::Bool(b) => write!(f, "boolean `{}`", b),
            Unexpected::Integer(ref i) => write!(f, "integer `{}`", i),
            Unexpected::Float(v) => write!(f, "floating point `{}`", v),
            Unexpected::Str(ref s) => write!(f, "string {:?}", s),
            Unexpected::Unit => write!(f, "unit value"),
            Unexpected::Array => write!(f, "array"),
            Unexpected::Map => write!(f, "map"),
        }
    }
}

pub struct Error {
    inner: Box<ErrorImpl>,
}

pub type Result<T, E = Error> = result::Result<T, E>;

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Io(source)),
        }
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl Error {
    #[doc(hidden)]
    #[cold]
    pub(crate) fn io(source: io::Error) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Io(source)),
        }
    }

    #[doc(hidden)]
    #[cold]
    pub(crate) fn too_large<T: Into<BigInt>>(index: T) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Range(index.into())),
        }
    }

    #[doc(hidden)]
    #[cold]
    pub(crate) fn invalid_type(unexpected: Unexpected, expected: &'static str) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Type {
                unexpected,
                expected,
            }),
        }
    }

    #[doc(hidden)]
    #[cold]
    pub(crate) fn path_parse<R: pest::RuleType + Send + Sync + 'static>(
        source: pest::error::Error<R>,
        path: &str,
    ) -> Self {
        Error {
            inner: Box::new(ErrorImpl::PathParse {
                path: Box::from(path),
                source: Box::new(source),
            }),
        }
    }

    #[doc(hidden)]
    #[cold]
    pub(crate) fn format_parse<E>(origin: &str, source: E) -> Self
    where
        E: StdError + Sync + Send + 'static,
    {
        Error {
            inner: Box::new(ErrorImpl::FormatParse {
                origin: origin.into(),
                source: Box::new(source),
            }),
        }
    }

    #[doc(hidden)]
    #[cold]
    pub(crate) fn serde<T: AsRef<str>>(message: T) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Serde(message.as_ref().into())),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&*self.inner, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self.inner {
            ErrorImpl::PathParse { ref source, .. } => Some(source.as_ref()),

            ErrorImpl::FormatParse { ref source, .. } => Some(source.as_ref()),

            _ => None,
        }
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Serde(msg.to_string().into())),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error {
            inner: Box::new(ErrorImpl::Serde(msg.to_string().into())),
        }
    }
}

/// Represents all possible errors that can occur when working with
/// configuration.
enum ErrorImpl {
    /// Value path could not be parsed (Origin Path).
    PathParse {
        path: Box<str>,
        source: Box<dyn StdError + Send + Sync>,
    },

    /// Some IO error occurred while file operation.
    Io(io::Error),

    /// Value could not be converted into the requested type.
    Type {
        /// What we found when parsing the value
        unexpected: Unexpected,

        /// What was expected when parsing the value
        expected: &'static str,
    },

    /// Value could not be parsed by target format.
    FormatParse {
        origin: Box<str>,
        source: Box<dyn StdError + Send + Sync>,
    },

    /// Serde error
    Serde(Box<str>),

    /// Path index over range
    Range(BigInt),
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorImpl::PathParse {
                ref path,
                ref source,
            } => write!(f, "{}\n for {}", source, path),

            ErrorImpl::Range(ref i) => write!(f, "invalid range {}", i),

            ErrorImpl::Type {
                ref unexpected,
                expected,
            } => {
                write!(f, "invalid type: {}, expected {}", unexpected, expected)?;
                Ok(())
            }

            ErrorImpl::Io(ref err) => write!(f, "{}", err),

            ErrorImpl::Serde(ref s) => write!(f, "{}", s),

            ErrorImpl::FormatParse {
                ref source,
                ref origin,
            } => {
                write!(f, "{}\n in {}", source, origin)?;
                Ok(())
            }
        }
    }
}
