use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display, Formatter};

/// `Minutes` is a newtype wrapper around u32 for explicitness.
/// Only used as an indicator rather than a full type.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minutes(pub u32);

impl<'de> Deserialize<'de> for Minutes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a u32 like would normally be done.
        let num: u32 = Deserialize::deserialize(deserializer)?;
        Ok(Minutes(num))
    }
}

impl Serialize for Minutes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

// Wrap around u32::fmt
impl Display for Minutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Newtype wrapping u64 for Unix Timestamp.
/// Only used as an indicator rather than a full type.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixTimestamp(pub u64);

impl<'de> Deserialize<'de> for UnixTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num: u64 = Deserialize::deserialize(deserializer)?;
        Ok(UnixTimestamp(num))
    }
}

impl Serialize for UnixTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.0)
    }
}

// Wrap around u64::fmt
impl Display for UnixTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
