use std::collections::HashMap;
use url::{ParseError, Url};

use crate::options::{Filter, ReviewType};

const STEAM_REV_API: &'static str = "https://store.steampowered.com/appreviews/";

/// State information/builder for the Steam review A.P.I.
///
/// https://partner.steamgames.com/doc/store/getreviews
#[derive(Debug)]
pub struct ReviewApi<'c> {
    /// Stores query pairs as key, value to parse with the url crate.
    query: HashMap<&'static str, &'c str>,
    /// Steam product's `appid`. May be found on each store page.
    appid: u32,
    /// Cursor to facilitate pagination.
    cursor: &'c str,
}

impl<'c> ReviewApi<'c> {
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
    /// let mut builder = ReviewApi::new(413410);
    /// ```
    pub fn new(appid: u32) -> Self {
        let mut api = Self {
            query: HashMap::new(),
            appid,
            cursor: "*",
        };

        // Set defaults
        let (key_json, val_json) = ReviewApi::add_json();
        api.query.insert(key_json, val_json);
        let (key_lang, val_lang) = ReviewApi::add_language();
        api.query.insert(key_lang, val_lang);
        // Default to querying by recency for pagination.
        api.filter(Filter::Recent);

        api
    }

    /// Change the builder's `appid`.
    ///
    /// See: the [`new()`] method for a description for `appid`.
    ///
    /// ## Warning
    /// This function **does not** check if `new_appid` is valid.
    /// Thus, a non-existing `appid` would only fail when the URL is requested.
    ///
    /// ## Examples
    /// ```rust
    /// let mut builder = ReviewApi::new(379720);
    /// builder.appid(2280);
    /// ```
    ///
    /// ```rust
    /// let mut builder = ReviewApi::new(460790);
    /// // 0 is an invalid appid, but we don't check!
    /// builder.appid(0);
    /// ```
    pub fn appid(&mut self, new_appid: u32) -> &mut Self {
        self.appid = new_appid;
        self
    }

    /// Return JSON from the API. Not settable by callers.
    fn add_json() -> (&'static str, &'static str) {
        ("json", "1")
    }

    /// Request reviews in a specific language. Currently not settable via my implementation.
    fn add_language() -> (&'static str, &'static str) {
        ("language", "english")
    }

    /// Return results in a specific order such as by most recent.
    ///
    /// The Steam API allows requesting review data in a specific order such as most helpful,
    /// recency, or last updated. Valve recommends `Filter::Recent` or `Filter::Updated`
    /// for pagination.
    ///
    /// The default is `Filter::All` as per the API. Leaving this unset will default to
    /// `Filter::Recent` to help pagination, however.
    pub fn filter(&mut self, filt: Filter) -> &mut Self {
        use Filter::*;
        self.query.entry("filter").insert(filt.as_str());
        self
    }

    pub fn review_type(&mut self, rev_type: ReviewType) -> &mut Self {
        use ReviewType::*;
        self.query.entry("review_type").insert(rev_type.as_str());
        self
    }

    pub fn build(&self) -> Result<Url, ParseError> {
        // STEAM_REV_API is valid so this shouldn't fail.
        let steam_base = Url::parse(STEAM_REV_API)
            .expect("Unexpected: Steam A.P.I. URL should parse correctly.");
        let app_id = self.appid.to_string();
        // ReviewApi.appid is a u32 so converting it to a String and joining appid to steam_base can't fail.
        let base_query = Url::join(&steam_base, &app_id)
            .expect("Unexpected: Joining the Steam A.P.I. and App ID should succeed.");

        Ok(Url::parse_with_params(
            &base_query.as_str(),
            self.query.iter(),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
