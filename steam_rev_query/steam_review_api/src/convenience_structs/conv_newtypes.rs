use serde::{Deserialize, Deserializer};
use std::fmt::{self, Display, Formatter};

/// `Hours` is a newtype wrapper around u32 for explicitness.
/// Only used as an indicator rather than a full type.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hours(u32);

impl<'de> Deserialize<'de> for Hours {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a u32 like would normally be done.
        let num: u32 = Deserialize::deserialize(deserializer)?;
        Ok(Hours(num))
    }
}

// Wrap around u32::fmt
impl Display for Hours {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Newtype wrapping u64 for Unix Timestamp.
/// Only used as an indicator rather than a full type.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixTimestamp(u64);

impl<'de> Deserialize<'de> for UnixTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num: u64 = Deserialize::deserialize(deserializer)?;
        Ok(UnixTimestamp(num))
    }
}

// Wrap around u64::fmt
impl Display for UnixTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
