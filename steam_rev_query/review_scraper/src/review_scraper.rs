use anyhow::{Context, Result};
use attohttpc::{self, RequestBuilder};
use std::{convert::TryFrom, time};
use steam_review_api::{
    convenience_structs::{flat_query::FlattenedQuery, SteamRevOuter},
    RevApiError, ReviewApi,
};

const fn user_agent() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
}

pub struct ReviewScraper<'a> {
    query: ReviewApi<'a>,
}

impl<'a> TryFrom<ReviewApi<'a>> for ReviewScraper<'a> {
    type Error = RevApiError;

    /// Build a ReviewScraper from a ReviewApi query.
    ///
    /// ## Example
    /// ```rust
    /// use steam_review_api::{Filter, ReviewApi, error::RevApiError};
    ///
    /// let query = ReviewApi::new(731490).build().unwrap();
    /// let mut scraper: ReviewScraper = query.try_into().unwrap();
    /// ```
    /// ## Errors
    /// ReviewScraper assumes that the caller wants pagination and thus
    /// returns `RevApiError::InvalidFilterCursor` for invalid states.
    fn try_from(query: ReviewApi<'a>) -> Result<Self, Self::Error> {
        if query.paging_ok() {
            Ok(Self { query })
        } else {
            Err(RevApiError::InvalidFilterCursor)
        }
    }
}

impl ReviewScraper<'_> {
    // Convenience function to build the internal query, send it, and receive
    // the response.
    // Building the query and parsing the JSON shouldn't fail.
    // Send might, though.
    fn send_request(&self) -> Result<SteamRevOuter> {
        // Wait here later.
        Ok(attohttpc::get(self.query.build().with_context(|| format!("Building the query failed which means something internal broke. Here's the entire ReviewApi struct! {:?}", self.query))?)
            .header("User-Agent", user_agent())
            .send().context("Failed sending the built SteamRevApi request.")?
            .json::<SteamRevOuter>()?)
    }

    pub fn pull(&mut self) -> Result<FlattenedQuery> {
        let raw_query = self.send_request()?;
        self.query.change_cursor(raw_query.cursor)?;
        Ok(raw_query.into())
    }
}
