//! The typed, validated values the CLI flags and the config file share: one
//! validator per value, called by both clap (`FromStr`) and serde.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The most memory one search or reference run may be asked for, in MB.
const MOST_MEMORY_MB: usize = 1 << 20;
/// The most checking threads one command may start.
const MOST_WORKERS: usize = 1 << 10;
/// The most repeats one bench may run per cell.
const MOST_REPEATS: usize = 1 << 10;

/// A flag's or a config key's value failed its one validation.
#[derive(Debug, PartialEq)]
pub(crate) struct ValueError(pub(crate) String);

impl fmt::Display for ValueError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        out.write_str(&self.0)
    }
}

impl std::error::Error for ValueError {}

/// A finite, positive number of seconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Seconds(pub f64);

impl TryFrom<f64> for Seconds {
    type Error = ValueError;
    fn try_from(seconds: f64) -> Result<Self, ValueError> {
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err(ValueError(format!(
                "must be finite and positive, not {seconds}"
            )));
        }
        if Duration::try_from_secs_f64(seconds).is_err() {
            return Err(ValueError(format!("is too large: {seconds}")));
        }
        Ok(Seconds(seconds))
    }
}

impl FromStr for Seconds {
    type Err = ValueError;
    fn from_str(value: &str) -> Result<Self, ValueError> {
        let seconds: f64 = value
            .parse()
            .map_err(|_| ValueError("needs a finite positive number of seconds".into()))?;
        Seconds::try_from(seconds)
    }
}

impl<'de> Deserialize<'de> for Seconds {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = f64::deserialize(deserializer)?;
        Seconds::try_from(raw).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Seconds {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

/// A positive integer bounded above, with `too`'s message over the bound.
macro_rules! bounded {
    ($name:ident($inner:ty), $range:expr, $needed:literal) => {
        impl TryFrom<i64> for $name {
            type Error = ValueError;
            fn try_from(number: i64) -> Result<Self, ValueError> {
                if !(1..=$range.max()).contains(&number) {
                    return Err(ValueError($range.message(number)));
                }
                Ok($name(number as $inner))
            }
        }

        impl FromStr for $name {
            type Err = ValueError;
            fn from_str(value: &str) -> Result<Self, ValueError> {
                let number: i64 = value.parse().map_err(|_| ValueError($needed.into()))?;
                $name::try_from(number)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = i64::deserialize(deserializer)?;
                $name::try_from(raw).map_err(serde::de::Error::custom)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_u64(self.0 as u64)
            }
        }
    };
}

/// A bound's error message for a number outside it.
trait Bound: Copy {
    fn message(self, number: i64) -> String;
    fn max(self) -> i64;
}

/// Ranges say `1..=MAX` plus a unit suffix; caps say `at most MAX`; a bare
/// minimum says `too large` over the type's own maximum.
#[derive(Clone, Copy)]
struct Ranged(usize, &'static str);
#[derive(Clone, Copy)]
struct AtMost(usize);
#[derive(Clone, Copy)]
struct AtLeast(u64);

impl Bound for Ranged {
    fn message(self, number: i64) -> String {
        format!("is 1..={}{}, not {number}", self.0, self.1)
    }
    fn max(self) -> i64 {
        self.0 as i64
    }
}

impl Bound for AtMost {
    fn message(self, number: i64) -> String {
        if number < 1 {
            format!("must be at least 1, not {number}")
        } else {
            format!("is at most {}, not {number}", self.0)
        }
    }
    fn max(self) -> i64 {
        self.0 as i64
    }
}

impl Bound for AtLeast {
    fn message(self, number: i64) -> String {
        if number < 1 {
            format!("must be at least 1, not {number}")
        } else {
            format!("is too large: {number}")
        }
    }
    fn max(self) -> i64 {
        self.0 as i64
    }
}

/// A memory label in MB, within the range the search supports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Megabytes(pub usize);
bounded!(
    Megabytes(usize),
    Ranged(MOST_MEMORY_MB, " MB"),
    "needs a positive integer number of megabytes"
);

/// The checks to run at a time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Workers(pub usize);
bounded!(
    Workers(usize),
    Ranged(MOST_WORKERS, ""),
    "needs a positive integer"
);

/// The bench runs per cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Repeats(pub usize);
bounded!(
    Repeats(usize),
    AtMost(MOST_REPEATS),
    "needs a positive integer"
);

/// The child evaluations a search may run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Evals(pub u64);
bounded!(
    Evals(u64),
    AtLeast(i64::MAX as u64),
    "needs a positive integer"
);

/// Where `line --pgn` stops the tree, in attacker moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Until(pub u32);
bounded!(
    Until(u32),
    AtLeast(u32::MAX as u64),
    "needs a positive integer"
);

/// Which move-ordering guide `prove` uses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Guide {
    Hand,
    None,
}

impl TryFrom<String> for Guide {
    type Error = ValueError;
    fn try_from(name: String) -> Result<Self, ValueError> {
        match name.as_str() {
            "hand" => Ok(Guide::Hand),
            "none" => Ok(Guide::None),
            _ => Err(ValueError(format!("is hand or none, not {name}"))),
        }
    }
}

impl FromStr for Guide {
    type Err = ValueError;
    fn from_str(value: &str) -> Result<Self, ValueError> {
        Guide::try_from(value.to_owned())
    }
}

impl<'de> Deserialize<'de> for Guide {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Guide::try_from(raw).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Guide {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Guide::Hand => "hand",
            Guide::None => "none",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds() {
        assert_eq!("1.5".parse(), Ok(Seconds(1.5)));
        for bad in ["x", "NaN", "inf", "0", "-1", "1e300"] {
            assert!(
                "1.5".parse::<Seconds>().is_ok() && bad.parse::<Seconds>().is_err(),
                "{bad}"
            );
        }
        assert_eq!(
            "0".parse::<Seconds>(),
            Err(ValueError("must be finite and positive, not 0".into()))
        );
        assert!(
            "1e300"
                .parse::<Seconds>()
                .unwrap_err()
                .0
                .starts_with("is too large:")
        );
    }

    #[test]
    fn megabytes() {
        assert_eq!("192".parse(), Ok(Megabytes(192)));
        assert_eq!(
            "0".parse::<Megabytes>(),
            Err(ValueError("is 1..=1048576 MB, not 0".into()))
        );
        assert!("-1".parse::<Megabytes>().is_err());
        assert!("1048577".parse::<Megabytes>().is_err());
        assert!("x".parse::<Megabytes>().is_err());
    }

    #[test]
    fn workers() {
        assert_eq!("4".parse(), Ok(Workers(4)));
        assert!("0".parse::<Workers>().is_err());
        assert!("1025".parse::<Workers>().is_err());
    }

    #[test]
    fn repeats() {
        assert_eq!("3".parse(), Ok(Repeats(3)));
        assert_eq!(
            "0".parse::<Repeats>(),
            Err(ValueError("must be at least 1, not 0".into()))
        );
        assert_eq!(
            "2000".parse::<Repeats>(),
            Err(ValueError("is at most 1024, not 2000".into()))
        );
    }

    #[test]
    fn evals() {
        assert_eq!("1000".parse(), Ok(Evals(1000)));
        assert!("0".parse::<Evals>().is_err());
        assert!("-1".parse::<Evals>().is_err());
    }

    #[test]
    fn until() {
        assert_eq!("7".parse(), Ok(Until(7)));
        assert!("0".parse::<Until>().is_err());
        assert!("99999999999".parse::<Until>().is_err());
    }

    #[test]
    fn guide() {
        assert_eq!("hand".parse(), Ok(Guide::Hand));
        assert_eq!("none".parse(), Ok(Guide::None));
        assert_eq!(
            "model".parse::<Guide>(),
            Err(ValueError("is hand or none, not model".into()))
        );
    }
}
