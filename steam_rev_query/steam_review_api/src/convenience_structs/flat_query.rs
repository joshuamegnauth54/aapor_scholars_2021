use super::{conv_newtypes::*, query_structs::Review};
use crate::language::Language;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct FlattenedQuery {
    pub recommendation_id: u64,
    pub steam_id: u64,
    pub num_games_owned: u32,
    pub num_reviews: u32,
    pub playtime_forever: Minutes,
    pub language: Language,
    pub review: String,
    pub timestamp_created: UnixTimestamp,
    pub voted_up: bool,
    pub votes_up: u32,
    pub votes_funny: u32,
    pub comment_count: u32,
    pub steam_purchase: bool,
    pub received_for_free: bool,
    pub written_during_early_access: bool,
    pub developer_response: Cow<'static, str>,
}

impl From<Review> for FlattenedQuery {
    fn from(other: Review) -> Self {
        Self {
            recommendation_id: other.recommendationid,
            steam_id: other.author.steamid,
            num_games_owned: other.author.num_games_owned,
            num_reviews: other.author.num_reviews,
            playtime_forever: other.author.playtime_forever,
            language: other.language,
            review: other.review,
            timestamp_created: other.timestamp_created,
            voted_up: other.voted_up,
            votes_up: other.votes_up,
            votes_funny: other.votes_funny,
            comment_count: other.comment_count,
            steam_purchase: other.steam_purchase,
            received_for_free: other.received_for_free,
            written_during_early_access: other.written_during_early_access,
            developer_response: other.developer_response.map_or("".into(), Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convenience_structs::ReviewAuthor;

    #[test]
    fn test_from() {
        let fake_rev = Review {
            recommendationid: 0,
            author: ReviewAuthor {
                steamid: 0,
                num_games_owned: 1337,
                num_reviews: 5,
                playtime_forever: Minutes(9001),
                playtime_last_two_weeks: Minutes(480),
                playtime_at_review: Some(Minutes(4096)),
                last_played: UnixTimestamp(1463112000),
            },
            language: Language::English,
            review: "😻😻😻 This game is full of CATS.".to_owned(),
            timestamp_created: UnixTimestamp(1618826641),
            timestamp_updated: UnixTimestamp(1618826641),
            voted_up: true,
            votes_up: 28,
            votes_funny: 54,
            weighted_vote_score: 0.0,
            comment_count: 9,
            steam_purchase: true,
            received_for_free: false,
            written_during_early_access: false,
            developer_response: None,
            timestamp_dev_responded: None,
        };

        let flattened: FlattenedQuery = fake_rev.into();
    }
}
