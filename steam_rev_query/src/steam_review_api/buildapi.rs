use std::collections::HashMap;
use url::{ParseError, Url};

use super::reviewtype::ReviewType;

const STEAM_REV_API: &'static str = "https://store.steampowered.com/appreviews/";

#[derive(Debug)]
pub struct ReviewApi<'c> {
    query: HashMap<&'static str, &'c str>,
    appid: u32,
    cursor: &'c str,
}

impl<'c> ReviewApi<'c> {
    pub fn new(appid: u32) -> Self {
        let mut api = Self {
            query: HashMap::new(),
            appid,
            cursor: "*",
        };

        let (key_json, val_json) = ReviewApi::add_json();
        api.query.insert(key_json, val_json);
        let (key_lang, val_lang) = ReviewApi::add_language();
        api.query.insert(key_lang, val_lang);

        api
    }

    fn appid(&mut self, new_appid: u32) -> &mut Self {
        self.appid = new_appid;
        self
    }

    fn add_json() -> (&'static str, &'static str) {
        ("json", "1")
    }

    fn add_language() -> (&'static str, &'static str) {
        ("language", "english")
    }

    pub fn review_type(&mut self, filter: ReviewType) -> &mut Self {
        use ReviewType::*;
        self.query.entry("review_type").insert(match filter {
            All => "all",
            Positive => "positive",
            Negative => "negative",
        });
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

fn add_filter() {}
