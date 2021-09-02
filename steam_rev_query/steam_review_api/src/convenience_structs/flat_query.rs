use super::{conv_newtypes::*, query_structs::Review};
use crate::language::Language;
use either::Either;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    rc::Rc,
};

/// TitleSerde is a reference counted String or a default in order to save memory.
/// The struct implements Serialize and Deserialize based on the value of Rc<String> or the
/// stored &'static str.
#[derive(Clone, Debug, Eq)]
pub struct TitleSerde(Either<Rc<str>, &'static str>);

impl TitleSerde {
    #[inline]
    pub fn is_default(&self) -> bool {
        self.as_ref() == "NA"
    }
}

// Serde traits
impl Serialize for TitleSerde {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Either::Left(title) => serializer.serialize_str(&*title),
            Either::Right(na_title) => serializer.serialize_str(na_title),
        }
    }
}

impl<'de> Deserialize<'de> for TitleSerde {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let title: String = Deserialize::deserialize(deserializer)?;
        Ok(TitleSerde(Either::Left(title.into())))
    }
}

// From<T> implementations
impl From<Rc<str>> for TitleSerde {
    #[inline]
    fn from(title: Rc<str>) -> Self {
        TitleSerde(Either::Left(title))
    }
}

impl From<&Rc<str>> for TitleSerde {
    #[inline]
    fn from(title: &Rc<str>) -> Self {
        TitleSerde(Either::Left(title.clone()))
    }
}

impl From<&'static str> for TitleSerde {
    #[inline]
    fn from(null_title: &'static str) -> Self {
        TitleSerde(Either::Right(null_title))
    }
}

impl From<String> for TitleSerde {
    #[inline]
    fn from(title: String) -> Self {
        TitleSerde(Either::Left(title.into()))
    }
}

// Misc traits
impl Display for TitleSerde {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Either::Left(title) => title.fmt(f),
            Either::Right(null_title) => null_title.fmt(f),
        }
    }
}

impl Default for TitleSerde {
    #[inline]
    fn default() -> Self {
        TitleSerde(Either::Right("NA"))
    }
}

impl AsRef<str> for TitleSerde {
    fn as_ref(&self) -> &str {
        match &self.0 {
            Either::Left(title) => &*title,
            Either::Right(null_title) => null_title,
        }
    }
}

// I'd want hashes on the string value level rather than on Either::Left and Either::Right
// along with the string. In other words, deriving Hash would produce two different hashes for
// Either::Left(Rc::new("meow".to_string()))
// and
// Either::Right("meow")
// (I checked.)
// Note: This behavior isn't wrong but I consider it wrong for my specific use case.
impl Hash for TitleSerde {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match &self.0 {
            Either::Left(title) => title.hash(hasher),
            Either::Right(null_title) => null_title.hash(hasher),
        }
    }
}

// Clippy yelled at me for deriving PartialEq.
impl PartialEq<TitleSerde> for TitleSerde {
    fn eq(&self, other: &TitleSerde) -> bool {
        // Hashes are implemented on the string themselves so I'll implement eq the same way.
        self.as_ref() == other.as_ref()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct FlattenedQuery {
    pub title: TitleSerde,
    pub appid: TitleSerde,
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
            title: TitleSerde::default(),
            appid: TitleSerde::default(),
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

impl FlattenedQuery {
    pub fn from_with_title_strs(other: Review, title: Rc<str>, appid: Rc<str>) -> Self {
        let mut query: Self = other.into();
        query.title = title.into();
        query.appid = appid.into();
        query
    }

    pub fn from_with_titles(other: Review, title: TitleSerde, appid: TitleSerde) -> Self {
        let mut query: Self = other.into();
        query.title = title;
        query.appid = appid;
        query
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

        let _flattened: FlattenedQuery = fake_rev.into();
    }
}
