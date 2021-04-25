use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// Steam review class (i.e. Overwhelmingly Positive) as an enum.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum ReviewScore {
    OverwhelminglyNegative = 1,
    VeryNegative,
    Negative,
    MostlyNegative,
    Mixed,
    MostlyPositive,
    Positive,
    VeryPositive,
    OverwhelminglyPositive,
}

// Unit struct for FromStr::Error.
// Users should never see this unless Steam changes their review levels.
// In that case I'd totally update these of course.
#[derive(Debug, Clone, Copy)]
pub struct ReviewScoreParseError;

impl std::error::Error for ReviewScoreParseError {}

impl Display for ReviewScoreParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "You should NOT see this error unless Valve changed their review descriptions. Please report this issue on GitHub.")
    }
}

impl ReviewScore {
    /// String representation of the review score.
    pub fn as_str(self) -> &'static str {
        use ReviewScore::*;
        match self {
            OverwhelminglyNegative => "Overwhelmingly Negative",
            VeryNegative => "Very Negative",
            Negative => "Negative",
            MostlyNegative => "Mostly Negative",
            Mixed => "Mixed",
            MostlyPositive => "Mostly Positive",
            Positive => "Positive",
            VeryPositive => "Very Positive",
            OverwhelminglyPositive => "Overwhelmingly Positive",
        }
    }
}

impl Display for ReviewScore {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ReviewScore {
    type Err = ReviewScoreParseError;

    /// Convert string slice to ReviewScore.
    ///
    /// You probably don't need to parse anything yourself.
    /// ReviewScore and the associated FromStr implementation are to help deserialization
    /// and save memory while doing so.
    ///
    /// ## Errors
    /// All nine of Steam's review classes are exhaustively covered by ReviewScore.
    /// Thus, parsing shouldn't cause an error unless:
    /// * the caller specifically parses a value not covered
    /// * Steam returns junk data somehow
    /// * Valve adds new review levels.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ReviewScore::*;
        match s {
            "Overwhelmingly Negative" => Ok(OverwhelminglyNegative),
            "Very Negative" => Ok(VeryNegative),
            "Negative" => Ok(Negative),
            "Mostly Negative" => Ok(MostlyNegative),
            "Mixed" => Ok(Mixed),
            "Mostly Positive" => Ok(MostlyPositive),
            "Positive" => Ok(Positive),
            "Very Positive" => Ok(VeryPositive),
            "Overwhelmingly Positive" => Ok(OverwhelminglyPositive),
            _ => Err(ReviewScoreParseError),
        }
    }
}

impl<'de> Deserialize<'de> for ReviewScore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        value.parse::<ReviewScore>().map_err(D::Error::custom)
    }
}

impl Serialize for ReviewScore {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
