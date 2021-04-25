use super::{conv_newtypes::*, reviewscore::ReviewScore};
use crate::language::Language;
use serde::{de::Error, Deserialize, Deserializer};

// Structs based on: https://partner.steamgames.com/doc/store/getreviews

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
    #[serde(deserialize_with = "str_to_uint")]
    pub steamid: u64,
    /// Reviewer's total amount of owned titles.
    pub num_games_owned: u32,
    /// Reviewer's total posted reviews.
    pub num_reviews: u32,
    /// Amount of minutes the reviewer played this title.
    /// May be zero for DLC.
    pub playtime_forever: Minutes,
    /// Amount of minutes reviewer played during the last two weeks.
    pub playtime_last_two_weeks: Minutes,
    /// Amount of minutes reviewer played at the moment they posted their review.
    /// NOTE: Some Steam releases are missing this field for some reason.
    pub playtime_at_review: Option<Minutes>,
    /// Unix timestamp indicating when the reviewer last played this title.
    pub last_played: UnixTimestamp,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
pub struct Review {
    /// Unique ID of the recommendation.
    #[serde(deserialize_with = "str_to_uint")]
    pub recommendationid: u64,
    /// Reviewer (Steam user).
    pub author: ReviewAuthor,
    /// Review's language.
    pub language: Language,
    /// Text of the review.
    pub review: String,
    /// Date the review was created as a Unix timestamp.
    pub timestamp_created: UnixTimestamp,
    /// Date the review was last updated as a Unix timestamp.
    /// Equal to `timestamp_created` if the author never updated the review.
    pub timestamp_updated: UnixTimestamp,
    /// Bool indicating if the review was positive (true) or negative (false).
    pub voted_up: bool,
    /// Number of users who voted this review up.
    pub votes_up: u32,
    /// Number of users who voted this review as funny.
    pub votes_funny: u32,
    /// Helpfulness score as calculated by Valve.
    #[serde(deserialize_with = "wvs_to_float")]
    pub weighted_vote_score: f64,
    /// Number of comments on this review.
    pub comment_count: u32,
    /// Did the reviewer purchase this game on Steam? True if yes.
    pub steam_purchase: bool,
    /// Was this game given to the reviewer for free? True if yes.
    /// (The user indicates this via a checkbox.)
    pub received_for_free: bool,
    /// Was this review written while the game was in Early Access? True if yes.
    pub written_during_early_access: bool,
    /// Text of the developer's response, if any.
    pub developer_response: Option<String>,
    /// Unix timestamp of the developer's response, if any.
    pub timestamp_dev_responded: Option<UnixTimestamp>,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
pub struct SteamRevOuter {
    /// Did the query succeed? NOTE: Don't rely on this to actually indicate success.
    #[serde(deserialize_with = "success_to_bool")]
    pub success: bool,
    /// Summary of current query such as how many reviews were pulled.
    pub query_summary: ReviewQuerySum,
    /// The `cursor` references the next page of information. Pass `cursor` into
    /// [ReviewApi::change_cursor] to paginate your current query.
    pub cursor: String,
    /// Reviews scraped.
    pub reviews: Vec<Review>,
}

// Converts Steam ID and recommendation ID from Strings to u64s.
// This should be fine as long as the API keeps returning
// u64s.
fn str_to_uint<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer)
        .and_then(|value: String| value.parse::<u64>().map_err(D::Error::custom))
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
    use super::*;

    // Test the whole dang thing.
    #[test]
    fn deserialize_all() {
        // Not real data. Based on actual output though.
        let pretend_reviews = r#"
        {
            "success": 1,
            "query_summary": {
                "num_reviews": 2,
                "review_score": 9,
                "review_score_desc": "Overwhelmingly Positive",
                "total_positive": 1337,
                "total_negative": 2,
                "total_reviews": 1339
            },
            "reviews": [
                {
                    "recommendationid": "0",
                    "author": {
                        "steamid": "0",
                        "num_games_owned": 1,
                        "num_reviews": 314,
                        "playtime_forever": 14284256,
                        "playtime_last_two_weeks": 20140,
                        "playtime_at_review": 1052950,
                        "last_played": 1618235983
                    },
                    "language": "english",
                    "review": "This game is so amazing all I do is play it except for writing this review 😹 meow",
                    "timestamp_created": 1618161312,
                    "timestamp_updated": 1618161312,
                    "voted_up": true,
                    "votes_up": 400,
                    "votes_funny": 114,
                    "weighted_vote_score": "0.56257367",
                    "comment_count": 6,
                    "steam_purchase": true,
                    "received_for_free": false,
                    "written_during_early_access": false
                },
                {
                    "recommendationid": "0",
                    "author": {
                        "steamid": "0",
                        "num_games_owned": 1400,
                        "num_reviews": 428,
                        "playtime_forever": 65248241,
                        "playtime_last_two_weeks": 10300,
                        "playtime_at_review": 5385919,
                        "last_played": 1554491710
                    },
                    "language": "english",
                    "review": "10/10 great combat great story pretty graphics meme123 ⭐⭐⭐⭐⭐",
                    "timestamp_created": 1585055110,
                    "timestamp_updated": 1585055110,
                    "voted_up": true,
                    "votes_up": 65,
                    "votes_funny": 2,
                    "weighted_vote_score": 0,
                    "comment_count": 1,
                    "steam_purchase": true,
                    "received_for_free": false,
                    "written_during_early_access": false
                }
            ],
            "cursor": "NOTAREALCURSOR"
        }
        "#;

        let _parsed: SteamRevOuter = serde_json::from_str(pretend_reviews).unwrap();
    }
}
