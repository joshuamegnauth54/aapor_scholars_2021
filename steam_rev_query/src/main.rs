use anyhow::Result;
use review_scraper::ReviewScraper;
use scraper_cache::ScraperCache;
use std::convert::TryInto;
use steam_review_api::{Filter, ReviewApi, ReviewType};

fn main() -> Result<()> {
    let mut steam = ReviewApi::new(1235140);
    steam
        .review_type(ReviewType::All)
        .filter(Filter::Recent)
        .unwrap()
        .num_per_page(100);

    let scraper: ReviewScraper = steam.try_into()?;

    Ok(())
}
