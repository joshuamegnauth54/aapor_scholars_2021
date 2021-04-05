use serde::{de::Error, Deserialize, Deserializer};
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
        write!(f, "You should NOT see this error unless Valve changed their review descriptions. Please report this issue.")
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
        let value: &str = Deserialize::deserialize(deserializer)?;
        value
            .parse::<ReviewScore>()
            .map_err(D::Error::custom)
    }
}
