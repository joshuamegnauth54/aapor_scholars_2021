mod arguments;
use arguments::ScraperAppSettings;
use rev_query_utils::error::Error;
use review_scraper::ReviewScraper;

use tracing::{info, warn};

fn main() {
    // Uses the RUST_LOG environmental variable like other loggers.
    tracing_subscriber::fmt::init();

    let test = ScraperAppSettings::from_arguments();
}
