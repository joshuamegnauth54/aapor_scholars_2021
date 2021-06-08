use steam_review_api::convenience_structs::{
    flat_query::{FlattenedQuery, TitleSerde},
    UnixTimestamp,
};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct ResumeInfo {
    pub appid: TitleSerde,
    pub timestamp: UnixTimestamp,
}

impl ResumeInfo {
    pub fn update(&mut self, query: &FlattenedQuery) -> Result<()> {
        // Update the timestamp if the query is older.
        if self.timestamp > query.timestamp_created {
            self.timestamp = query.timestamp_created;
        }

        // I only support resuming from one appid currently.
        // So replace the appid if it's null or fail if they're different.
        if self.appid.is_default() {
            self.appid = query.appid.clone();
            Ok(())
        } else if self.appid.as_ref() != query.appid.as_ref() {
            Err(Error::MultipleAppids)
        } else {
            // If the appids aren't different nor is self.appid == "NA" then
            // the query's appid and self.appid are the same.
            Ok(())
        }
    }
}

impl Default for ResumeInfo {
    #[inline]
    fn default() -> Self {
        Self {
            appid: TitleSerde::default(),
            timestamp: UnixTimestamp(u64::MAX),
        }
    }
}
