use super::{conv_newtypes::*, reviewscore::ReviewScore};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewQuerySum {
    num_reviews: u8,
    review_score: u8,
    review_score_desc: ReviewScore,
    total_positive: u32,
    total_negative: u32,
    total_reviews: u32,
}

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewAuthor {
    steamid: String,
    num_games_owned: u32,
    num_reviews: u32,
    // Play times are in hours.
    playtime_forever: Hours,
    playtime_last_two_weeks: Hours,
    playtime_at_review: Hours,
    last_played: UnixTimestamp,
}

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Review {
    recommendationid: String,
    author: ReviewAuthor,
    language: String,
    review: String,
}

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SteamRevOuter {
    /// Did the query succeed? NOTE: Don't rely on this to actually indicate success.
    #[serde(deserialize_with = "success_to_bool")]
    success: bool,
    query_summary: ReviewQuerySum,
    cursor: String,
    reviews: Vec<Review>,
}

// Converts the success field into a bool.
fn success_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value: u8 = Deserialize::deserialize(deserializer)?;
    if value == 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}
