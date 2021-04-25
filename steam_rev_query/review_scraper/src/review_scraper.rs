use anyhow::{Context, Result};
use std::{
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    time::{Duration, Instant},
};
use steam_review_api::{
    convenience_structs::{flat_query::FlattenedQuery, SteamRevOuter},
    RevApiError, ReviewApi,
};

const fn user_agent() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug, Clone, Copy)]
struct DumbTimer {
    last: Instant,
    fire_time: Duration,
}

impl DumbTimer {
    fn new(secs: u64) -> Self {
        Self {
            last: Instant::now(),
            fire_time: Duration::from_secs(secs),
        }
    }

    #[inline]
    fn elapsed(&self) -> Duration {
        Instant::now() - self.last
    }

    #[inline]
    fn time_left(&self) -> Duration {
        // Subtraction can overflow here, but returning Duration::ZERO is fine
        // anyway. So, I'm checking the subtraction just in case.
        self.fire_time.saturating_sub(self.elapsed())
    }

    // True if more time than fire_time passed.
    #[inline]
    fn complete(&self) -> bool {
        self.elapsed() >= self.fire_time
    }

    #[inline]
    fn reset(&mut self) {
        self.last = Instant::now();
    }

    fn wait_fire(&mut self) {
        if !self.complete() {
            std::thread::sleep(self.time_left());
        }
        self.reset();
    }
}

pub struct ReviewScraper {
    query: ReviewApi,
    timer: DumbTimer,
}

impl TryFrom<ReviewApi> for ReviewScraper {
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
    fn try_from(query: ReviewApi) -> Result<Self, Self::Error> {
        if query.paging_ok() {
            Ok(Self {
                query,
                timer: DumbTimer::new(30),
            })
        } else {
            Err(RevApiError::InvalidFilterCursor)
        }
    }
}

impl TryFrom<&ReviewApi> for ReviewScraper {
    type Error = RevApiError;

    fn try_from(query: &ReviewApi) -> Result<Self, Self::Error> {
        query.clone().try_into()
    }
}

impl TryFrom<&mut ReviewApi> for ReviewScraper {
    type Error = RevApiError;

    fn try_from(query: &mut ReviewApi) -> Result<Self, Self::Error> {
        query.clone().try_into()
    }
}

impl ReviewScraper {
    // Convenience function to build the internal query, send it, and receive
    // the response.
    // Building the query and parsing the JSON shouldn't fail.
    // Send might, though.
    fn send_request(&mut self) -> Result<SteamRevOuter> {
        self.timer.wait_fire();
        Ok(attohttpc::get(self.query.build().with_context(|| format!("Building the query failed which means something internal broke. Here's the entire ReviewApi struct! {:?}", self.query))?)
            .header("User-Agent", user_agent())
            .send().context("Failed sending the built SteamRevApi request.")?
            .json::<SteamRevOuter>()?)
    }

    pub fn pull<B>(&mut self) -> Result<B>
    where
        B: FromIterator<FlattenedQuery>,
    {
        let raw_query = self.send_request()?;
        // Update cursor for pagination.
        // This shouldn't fail because we checked if pagination is okay
        // when we built the Scraper.
        self.query.change_cursor(raw_query.cursor)?;
        Ok(raw_query
            .reviews
            .into_iter()
            .map(Into::<FlattenedQuery>::into)
            .collect())
    }
}

impl Iterator for ReviewScraper {
    // Figure out how to make this generic over
    // other collections later.
    type Item = Result<Vec<FlattenedQuery>>;

    fn next(&mut self) -> Option<Self::Item> {
        // I want to return None for the Iterator if the
        // query actually returns nothing.
        //
        // Without doing this the Iterator could produce
        // sequences of Some with empty Vectors which is dumb
        // and may cause problems too.
        match self.pull() {
            Result::<Vec<FlattenedQuery>>::Ok(query) => {
                if !query.is_empty() {
                    Some(Ok(query))
                } else {
                    None
                }
            }
            // Don't wanna discard errors even if this looks clumsy.
            // Transposing Err => None doesn't solve the problem above
            // but also discards errors. Losses all around.
            Err(e) => Some(Err(e)),
        }
    }
}
