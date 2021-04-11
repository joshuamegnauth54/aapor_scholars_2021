use super::{conv_newtypes::*, reviewscore::ReviewScore};
use crate::language::Language;
use serde::{Deserialize, Deserializer};

/// Summary of the query as a whole as well as data on the game such as total amount of reviews.
/// Only `num_reviews` is present across multiple queries. `review_score_desc` et cetera are only
/// available in the first query.
#[derive(Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewQuerySum {
    /// Number of reviews in this query.
    pub num_reviews: u8,
    // Review score is covered by the ReviewScore enum for both the number and the description.
    #[serde(skip)]
    review_score: u8,
    /// Wilson review score plus description (1-9 where 9 is the most positive).
    pub review_score_desc: Option<ReviewScore>,
    /// Total positive reviews (i.e. for the game as a whole on Steam not just the query).
    pub total_positive: Option<u32>,
    /// Total negative reviews (i.e. for the game as a whole on Steam not just the query).
    pub total_negative: Option<u32>,
    /// Total reviews present on Steam.
    pub total_reviews: Option<u32>,
}

#[derive(Debug, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewAuthor {
    /// Reviewer's SteamID64. See [this link](https://developer.valvesoftware.com/wiki/SteamID) for info.
    pub steamid: String,
    /// Reviewer's total amount of owned titles.
    pub num_games_owned: u32,
    /// Reviewer's total posted reviews.
    pub num_reviews: u32,
    /// Amount of minutes the reviewer played this title.
    pub playtime_forever: Minutes,
    /// Amount of minutes reviewer played during the last two weeks.
    pub playtime_last_two_weeks: Minutes,
    /// Amount of minutes reviewer played at the moment they posted their review.
    pub playtime_at_review: Minutes,
    /// Unix timestamp indicating when the reviewer last played this title.
    pub last_played: UnixTimestamp,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
pub struct Review {
    pub recommendationid: String,
    pub author: ReviewAuthor,
    pub language: Language,
    pub review: String,
    pub timestamp_created: UnixTimestamp,
    pub timestamp_updated: UnixTimestamp,
    pub voted_up: bool,
    pub votes_up: u32,
    pub votes_funny: u32,
    #[serde(deserialize_with = "wvs_to_float")]
    pub weighted_vote_score: f64,
    pub comment_count: u32,
    pub steam_purchase: bool,
    pub received_for_free: bool,
    pub written_during_early_access: bool,
    pub developer_response: Option<String>,
    pub timestamp_dev_responded: Option<UnixTimestamp>,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
pub struct SteamRevOuter {
    /// Did the query succeed? NOTE: Don't rely on this to actually indicate success.
    #[serde(deserialize_with = "success_to_bool")]
    pub success: bool,
    pub query_summary: ReviewQuerySum,
    pub cursor: String,
    pub reviews: Vec<Review>,
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
        // According to the documentation, 0 is false and 1 is true.
        // But, the success field is unreliable. I've personally tested
        // incorrect queries that returned nothing yet the API
        // returned 1.
        // Anyway, returning an error for non-binary results doesn't make
        // sense.
        Ok(false)
    }
}

// Converts weighted_vote_score (String) to f64
fn wvs_to_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    // Steam's API returns either a String or 0 for weighted_vote_score.
    // 0 is an integer rather than a float and the String holds a float.
    // But we want a f64 to save memory instead of a String of course.
    Deserialize::deserialize(deserializer)
        // Return Ok(0.) if deserialization fails (i.e. if we get a u8 instead)
        // of a String back.
        // Otherwise, pass on the Result from parse.
        .map_or(Ok(0.), |value: String| value.parse::<f64>())
        // Return the successfully parsed f64 OR 0.0 if it failed
        // for some reason. This is consistent with the API returning
        // 0 but as a f64 instead.
        .or(Ok(0.))
}

#[cfg(test)]
mod tests {

}
