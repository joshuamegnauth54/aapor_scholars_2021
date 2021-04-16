use std::{borrow::Cow, collections::HashMap, convert::TryFrom};
use url::{ParseError, Url};

use crate::{
    error::RevApiError,
    language::Language,
    options::{Filter, PurchaseType, ReviewType},
};

const STEAM_REV_API: &str = "https://store.steampowered.com/appreviews/";

/// State information/builder for the Steam review A.P.I.
///
/// https://partner.steamgames.com/doc/store/getreviews
#[derive(Debug)]
pub struct ReviewApi<'val> {
    /// Stores query pairs as key, value to parse with the url crate.
    query: HashMap<&'static str, Cow<'val, str>>,
    /// Steam product's `appid`. May be found on each store page.
    appid: u32,
}

impl<'val> ReviewApi<'val> {
    /// Construct a builder with an `appid`.
    ///
    /// Each product on Steam has an `appid` which is available via the associated
    /// Store page's URL. For example, [Overcooked! 2](https://store.steampowered.com/app/728880/Overcooked_2/)'s page is
    /// **https://store.steampowered.com/app/728880/Overcooked_2/**. The `appid` is 728880.
    ///
    /// ## Warning
    /// This function **does not** check if the provided `appid` is valid.
    ///
    /// ## Example
    /// ```rust
    /// use steam_review_api::ReviewApi;
    ///
    /// let mut builder = ReviewApi::new(413410);
    /// ```
    pub fn new(appid: u32) -> Self {
        let mut api = Self {
            query: HashMap::new(),
            appid,
        };

        // Set defaults
        let (key_json, val_json) = ReviewApi::add_json();
        api.query.insert(key_json, val_json.into());
        let (key_lang, val_lang) = ReviewApi::add_language(Language::English);
        api.query.insert(key_lang, val_lang.into());
        // Default to querying by recency for pagination.
        api.filter(Filter::Recent).expect(
            "Unexpected: Changing the filter to Recent shouldn't cause an error in the ctor.",
        );
        // Add default cursor
        api.change_cursor("*")
            .expect("Unexpected: Impossible to fail via an invalid Filter here.");

        api
    }

    #[inline]
    pub fn current_appid(&self) -> u32 {
        self.appid
    }

    /// Change the builder's `appid`. The `cursor` is reset as well.
    ///
    /// See: the [`new()`] method for a description for `appid`.
    ///
    /// ## Warning
    /// This function **does not** check if `new_appid` is valid.
    /// Thus, a non-existing `appid` would only fail when the URL is requested.
    ///
    /// ## Examples
    /// ```rust
    /// use steam_review_api::ReviewApi;
    ///
    /// let mut builder = ReviewApi::new(379720);
    /// builder.appid(2280);
    /// ```
    ///
    /// ```rust
    /// use steam_review_api::ReviewApi;
    ///
    /// let mut builder = ReviewApi::new(460790);
    /// // 0 is an invalid appid, but we don't check!
    /// builder.appid(0);
    /// ```
    pub fn appid(&mut self, new_appid: u32) -> &mut Self {
        self.appid = new_appid;
        self.query.entry("cursor").insert("*".into());
        self
    }

    /// Return JSON from the API. Not settable by callers.
    #[inline]
    const fn add_json() -> (&'static str, &'static str) {
        ("json", "1")
    }

    /// Request reviews in a specific language. Currently not settable via my implementation.
    fn add_language(lang: Language) -> (&'static str, &'static str) {
        ("language", lang.as_str())
    }

    /// Return results in a specific order such as by most recent.
    ///
    /// The Steam API allows requesting review data in a specific order such as most helpful,
    /// recency, or last updated. Valve recommends `Filter::Recent` or `Filter::Updated`
    /// for pagination. Setting a day range requires `Filter::All`.
    ///
    /// Valve's default is `Filter::All` as per the API. Leaving this unset will default to
    /// `Filter::Recent` to help pagination, however.
    ///
    /// ## Note on multiple calls
    /// This functions overwrites any previously set Filter.
    ///
    /// ## Errors
    /// Invalid API states, such trying to set a `Filter::All` when a Cursor exists,
    /// returns an error.
    pub fn filter(&mut self, filt: Filter) -> Result<&mut Self, RevApiError> {
        use Filter::*;
        use RevApiError::*;
        match filt {
            // "*" is the default cursor. Thus, fail on any other cursor for All.
            All if self.query.get("cursor").expect("Cursor is always present.") != "*" => {
                Err(InvalidFilterCursor)
            }
            // Only All is valid for day_range.
            Recent | Updated if self.query.contains_key("day_range") => Err(InvalidFilterDayRange),
            _ => {
                self.query.entry("filter").insert(filt.as_str().into());
                Ok(self)
            }
        }
    }

    pub fn day_range(&mut self, days_ago: u32) -> Result<&mut Self, RevApiError> {
        match &**self
            .query
            .get("filter")
            .expect("Unexpected: Filter is always set so you shouldn't see this message")
        {
            "all" => {
                self.query
                    .entry("day_range")
                    .insert(days_ago.to_string().into());
                Ok(self)
            }
            "recent" | "updated" => Err(RevApiError::InvalidFilterDayRange),
            bad => unreachable!(
                concat!(
                    "Unexpected: The stored query string for Filter can't be ",
                    "anything other than the variants. Got: {}"
                ),
                bad
            ),
        }
    }

    pub fn change_cursor(&mut self, new_cursor: &'val str) -> Result<&mut Self, RevApiError> {
        if self.paging_ok() {
            self.query.entry("cursor").insert(new_cursor.into());
            Ok(self)
        } else {
            Err(RevApiError::InvalidFilterCursor)
        }
    }

    pub fn review_type(&mut self, rev_type: ReviewType) -> &mut Self {
        // Note: ReviewType -> &'static str -> Cow
        self.query
            .entry("review_type")
            .insert(rev_type.as_str().into());
        self
    }

    pub fn purchase_type(&mut self, purchase: PurchaseType) -> &mut Self {
        // Note: PurchaseType -> &'static str -> Cow
        self.query
            .entry("purchase_type")
            .insert(purchase.as_str().into());
        self
    }

    /// Set a maximum number of results per page up to 100.
    ///
    /// ## Note on maximum
    /// This function does not raise an error if `amount` > 100.
    /// Steam's review API returns the maximum rather than fails if
    /// a large `num_per_page` is passed in.
    ///
    /// ## Default
    /// Defaults to 20 via the API if unset.
    ///
    /// ## Overwrite
    /// This function overwrites any previously set `num_per_page`.
    ///
    /// ## Example
    /// ```rust
    /// use steam_review_api::ReviewApi;
    ///
    /// let mut builder = ReviewApi::new(374320);
    /// builder.num_per_page(100);
    /// ```
    pub fn num_per_page(&mut self, amount: u8) -> &mut Self {
        self.query
            .entry("num_per_page")
            .insert(amount.to_string().into());
        self
    }

    ///
    pub fn paging_ok(&self) -> bool {
        match &**self
            .query
            .get("filter")
            .expect("Unexpected: Filter is always set so you shouldn't see this message.")
        {
            // I feel dirty matching on str. Implementing a function to convert &str to Filter seems
            // comparably worse though.
            "all" => false,
            "recent" | "updated" => true,
            // The API doesn't expose the internal HashMap so this shouldn't happen.
            // I could just match on _ and ignore "all", but I'd rather catch if this implodes somehow.
            bad => unreachable!(
                concat!(
                    "Unexpected: The stored query string for Filter can't be ",
                    "anything other than the variants. Got: {}"
                ),
                bad
            ),
        }
    }

    /// Build a query into a Url.
    ///
    /// ## Errors
    /// Internal parsing errors return a ParseError.
    /// Nothing should actually cause a ParseError, but the function
    /// returns `Result` just in case. ParseErrors should be reported.
    ///
    /// ## Example
    /// ```rust
    /// use steam_review_api::{PurchaseType, ReviewApi};
    ///
    /// let query = ReviewApi::new(227600)
    ///     .purchase_type(PurchaseType::All)
    ///     .num_per_page(100)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(&self) -> Result<Url, ParseError> {
        // STEAM_REV_API is valid so this shouldn't fail.
        let steam_base = Url::parse(STEAM_REV_API)
            .expect("Unexpected: Steam A.P.I. URL should parse correctly.");
        let app_id = self.appid.to_string();
        // ReviewApi.appid is a u32 so converting it to a String and joining appid to steam_base can't fail.
        let base_query = Url::join(&steam_base, &app_id)
            .expect("Unexpected: Joining the Steam A.P.I. and App ID should succeed.");

        Url::parse_with_params(&base_query.as_str(), self.query.iter())
    }
}

impl TryFrom<&ReviewApi<'_>> for Url {
    type Error = ParseError;

    fn try_from(value: &ReviewApi) -> Result<Url, Self::Error> {
        value.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_with_cursor() {
        let mut steam = ReviewApi::new(1235140);
        steam
            .review_type(ReviewType::All)
            .filter(Filter::Updated)
            .expect("Unexpected: Setting Filter::Recent.")
            .change_cursor("lol!meow@cats$")
            .expect("Unexpected: Filter is All for some reason?")
            .review_type(ReviewType::All)
            .purchase_type(PurchaseType::All);
        let _built_api = steam.build().expect("You broke build(), Josh.");
    }

    #[test]
    fn cursor_default_filter() {
        let _built_api = ReviewApi::new(21690)
            .change_cursor("koolfakecursor")
            .expect("Unexpected: Filter is All for some reason?")
            .build()
            .expect("Yay build() is broken now!");
    }

    #[test]
    fn cursor_filter_all() {
        let _built_api = ReviewApi::new(584400)
            .change_cursor("dontpanikherepls")
            .expect("Unexpected: Filter is All before I set it to All!!")
            .filter(Filter::All)
            .expect_err("Setting filter to All with a cursor didn't return an error.");
    }

    #[test]
    fn days_range_correct() {
        let _built_api = ReviewApi::new(311690)
            .filter(Filter::All)
            .expect("Unexpected: Changing the Filter right after constructing shouldn't raise an error.")
            .day_range(365)
            .expect("Filter is set to All yet day_range() failed.")
            .build().expect("I broke build().");
    }
}
