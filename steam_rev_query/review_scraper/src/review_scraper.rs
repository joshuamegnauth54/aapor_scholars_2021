use attohttpc::header::{COOKIE, USER_AGENT};
use lazy_static::lazy_static;
use rev_query_utils::error::{Error, Result};
use scraper::{Html, Selector};
use std::{
    convert::{TryFrom, TryInto},
    iter::FromIterator,
    time::{Duration, Instant},
};
use steam_review_api::{
    convenience_structs::{
        flat_query::{FlattenedQuery, TitleSerde},
        SteamRevOuter,
    },
    RevApiError, ReviewApi,
};
use tracing::{debug, error, info};

// This only works with Cargo so I'll need an alternative.
const fn user_agent() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
}

// App page on Steam. I need this to pull out the game's title because the API doesn't return the title.
// This is unlikely to change over time (I guess).
const STEAM_APP_PAGE: &str = "https://store.steampowered.com/app/";

// Every app page stores the name in this attribute:
// <div class=\"apphub_AppName\">NAME</div>
// Using the page's title doesn't work because Steam tacks on extras that I suck at parsing (such as for sales).
// Besides, pulling the appid's title directly from the element is likely better than extracting it with regular expressions.
const STEAM_APP_NAME: &str = r#"div[class="apphub_AppName"]"#;

// Steam redirects to an age gate for mature content (games rated Mature or higher or the equivalent for example).
// The cookies below are set to determine a user's age and whether Steam should show the content.
// Also...Unix's birthday 🥳
const AGE_GATE_COOKIE: &str =
    "birthtime=-2660399; path=/; lastagecheckage=1-0-1970; path=/; wants_mature_content=1; path=/";

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

    // Return how much time passed since the timer last fired.
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

    // Reset the timer to Instant::now since everything is implemented in relation to "now."
    #[inline]
    fn reset(&mut self) {
        self.last = Instant::now();
    }

    // Wait for at least how much time is left based on when we last fired the timer.
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
    // Title of the game (i.e. appid title).
    // I need to wrap this in an Rc because multiple FlattenedQuery structs
    // need to store the title. Cloning hundreds of these Strings would be wasteful,
    // and figuring out lifetimes where the cache, FlattenedQuerys, and Scraper are all
    // mutually dependent due to one String was very painful.
    app_title: TitleSerde,
    appid: TitleSerde,
}

impl TryFrom<ReviewApi> for ReviewScraper {
    type Error = Error;

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
    fn try_from(query: ReviewApi) -> Result<Self> {
        if query.paging_ok() {
            let (app_title, appid) = Self::try_fetch_title(query.current_appid());

            Ok(Self {
                query,
                timer: DumbTimer::new(30),
                app_title,
                appid,
            })
        } else {
            Err(RevApiError::InvalidFilterCursor.into())
        }
    }
}

impl TryFrom<&ReviewApi> for ReviewScraper {
    type Error = Error;

    #[inline]
    fn try_from(query: &ReviewApi) -> Result<Self> {
        query.clone().try_into()
    }
}

impl TryFrom<&mut ReviewApi> for ReviewScraper {
    type Error = Error;

    #[inline]
    fn try_from(query: &mut ReviewApi) -> Result<Self> {
        query.clone().try_into()
    }
}

impl ReviewScraper {
    // Convenience function to build the internal query, send it, and receive
    // the response.
    // Building the query and parsing the JSON shouldn't fail.
    // Send might, though.
    fn send_request(&mut self) -> Result<SteamRevOuter> {
        // Unfortunately, this will wait for the first request as well!
        self.timer.wait_fire();
        Ok(attohttpc::get(self.query.build()?)
            .header("User-Agent", user_agent())
            .send()?
            .json::<SteamRevOuter>()?)
    }

    pub fn pull<B>(&mut self) -> Result<B>
    where
        B: FromIterator<FlattenedQuery>,
    {
        let raw_query = self.send_request()?;
        info!(
            "Pulled data for {}({}).",
            self.appid.as_ref(),
            self.app_title.as_ref()
        );
        // Update cursor for pagination.
        // This shouldn't fail because we checked if pagination is okay when we built the Scraper.
        // (And either way using day_range is messy).
        self.query.change_cursor(raw_query.cursor, true)?;
        Ok(raw_query
            .reviews
            .into_iter()
            .map(|outer| {
                FlattenedQuery::from_with_titles(outer, self.app_title.clone(), self.appid.clone())
            })
            .collect())
    }

    #[inline]
    fn create_title_request(appid_str: &TitleSerde) -> attohttpc::RequestBuilder {
        attohttpc::get(format!("{}{}", STEAM_APP_PAGE, appid_str))
            .header(USER_AGENT, user_agent())
            // Cookies are needed to bypass Steam's age check page so I can scrape RIP AND TEAR!!!
            .header_append(COOKIE, AGE_GATE_COOKIE)
    }

    fn try_fetch_title(appid: u32) -> (TitleSerde, TitleSerde) {
        // NOTE: I forgot why I'm storing this as a TitleSerde...
        // I can probably just keep it as a u32.
        let appid_str = appid.to_string().into();
        info!("Attempting to scrape title for {}.", appid_str);

        // Cache the parsed selector in case we're fetching multiple appids.
        lazy_static! {
            static ref DIV_APPNAME: Selector =
                Selector::parse(STEAM_APP_NAME).expect("Known valid Selector failed to parse.");
        }

        // I don't need to prepare the request BUT doing so greatly eases debugging.
        let mut app_request = ReviewScraper::create_title_request(&appid_str).prepare();
        debug!("Headers for the get request: {:?}", app_request.headers());

        // Fetch the store page for appid.
        match app_request.send() {
            Ok(response) => {
                // Splitting the response in case I need to print some debug information.
                let (status_code, headers, response_reader) = response.split();

                match response_reader.text() {
                    Ok(raw_html) => {
                        // Parse HTML and then search for the class and element that holds the app name.
                        let html = Html::parse_document(&raw_html);
                        if let Some(title_ele) = html.select(&DIV_APPNAME).next() {
                            (title_ele.text().collect::<String>().into(), appid_str)
                        } else {
                            // NOT finding the app title means that the selector is wrong or changed.
                            // (Or Steam is redirecting to some page like the age gate that is not being bypassed).
                            error!(
                            "Failed to find a title for app {} in the store page's HTML. Please report this issue. Selector: {:?}",
                            appid_str,
                            *DIV_APPNAME
                        );
                            debug!("Headers received from Steam: {:?}", headers);
                            debug!("Status code: {}", status_code);
                            (TitleSerde::default(), appid_str)
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error converting HTML from Steam to text for appid {}. Error: {}",
                            appid_str, e
                        );
                        (TitleSerde::default(), appid_str)
                    }
                }
            }
            Err(e) => {
                error!("Error fetching title for appid {}. Error: {}", appid_str, e);
                (TitleSerde::default(), appid_str)
            }
        }
    }

    #[inline]
    pub fn title(&self) -> &str {
        self.app_title.as_ref()
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

#[cfg(test)]
mod tests {
    use super::*;

    // Appid and titles. These shouldn't ever change.
    const DOOM_2016: u32 = 379720;
    const DOOM_2016_TITLE: &'static str = "DOOM";
    const SYMPHONIA: u32 = 372360;
    const SYMPHONIA_TITLE: &'static str = "Tales of Symphonia";

    #[test]
    fn can_i_haz_title() {
        let (title_doom, _appid_doom) = ReviewScraper::try_fetch_title(DOOM_2016);
        assert_eq!(title_doom.as_ref(), DOOM_2016_TITLE);

        let (title_tales, _appid_tales) = ReviewScraper::try_fetch_title(SYMPHONIA);
        assert_eq!(title_tales.as_ref(), SYMPHONIA_TITLE);
    }
}
