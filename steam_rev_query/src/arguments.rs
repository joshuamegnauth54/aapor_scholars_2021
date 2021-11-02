use chrono::{offset, TimeZone, Utc};
use clap::{App, Arg, ArgMatches};
use either::Either;
use rev_query_utils::error::{Error, Result};
use review_scraper::ReviewScraper;
use scraper_cache::{ResumeScraperCache, ScraperCache};
use std::{
    convert::TryInto,
    io::ErrorKind,
    iter::{Map, Rev, Take},
};
use steam_review_api::{
    convenience_structs::flat_query::FlattenedQuery, Filter, ReviewApi, ReviewType,
};
use tracing::{info, warn};

const DEFAULT_CACHE_SIZE: usize = 500;
trait EndAfterZero = Fn(Result<Vec<FlattenedQuery>>) -> Result<Vec<FlattenedQuery>>;

fn end_after_zero_wrap(
    item: Result<Vec<FlattenedQuery>>,
    keep_going: bool,
) -> Result<Vec<FlattenedQuery>> {
    // Ugly. :(
    // Returns an empty vector if the scraper should keep going if all duplicates were received.
    match item {
        Ok(item) => Ok(item),
        Err(Error::NoDataAfterFiltering) if keep_going => Ok(vec![]),
        Err(e) => Err(e),
    }
}

pub(crate) struct ScraperAppSettings<IterMapFn>
where
    IterMapFn: EndAfterZero,
{
    pub scraper: Either<Map<Take<ReviewScraper>, IterMapFn>, Map<ReviewScraper, IterMapFn>>,
    pub cache: ScraperCache,
}

impl<IterMapFn> ScraperAppSettings<IterMapFn>
where
    IterMapFn: EndAfterZero,
{
    pub(crate) fn from_arguments() -> Self {
        let arg_matches = build_arguments();
        build_scraper(arg_matches)
    }
}

fn wrap_scraper<IterMapFn>(
    scraper: ReviewScraper,
    scrape_n: Option<usize>,
    end_after_zero: bool,
) -> Either<Map<Take<ReviewScraper>, IterMapFn>, Map<ReviewScraper, IterMapFn>>
where
    IterMapFn: EndAfterZero,
{
    if let Some(scrape_n) = scrape_n {
        Either::Left(
            scraper
                .take(scrape_n)
                .map(|item| end_after_zero_wrap(item, end_after_zero)),
        )
    } else {
        Either::Right(scraper.map(|item| end_after_zero_wrap(item, end_after_zero)))
    }
}

// Panic on IO errors with useful messages.
fn io_error_handler(error: ErrorKind, path: &str) -> ! {
    use ErrorKind::*;
    match error {
        NotFound => panic!("You need to pass in a valid file to resume from. Not found: {}", path),
        PermissionDenied => panic!("You don't have the required permissions for path: {}", path),
        AlreadyExists => panic!("You tried to start a NEW scrape but the file at path ({}) exists. Did you forget to pass --resume?", path),
        _ => panic!("Unspecified error: {:?}", error)
    }
}

fn build_arguments() -> ArgMatches<'static> {
    App::new("Steam User Reviews Scraper")
        .version("0.13")
        .author("Joshua Megnauth")
        .about("Scrape or resume a scrape of user reviews for a Steam appid.")
        .arg(
            Arg::with_name("OUTPUT")
            .help("Write scrape results to or resume from this file.")
            .required(true)
            .index(1)
        )
        .arg(
            Arg::with_name("appid")
                .short("a")
                .long("appid")
                .help("Steam appid to scrape. Find the appid via the Steam Store.")
                .takes_value(true)
                .required_unless("resume"),
        )
        .arg(
            Arg::with_name("review_type")
                 .short("t")
                 .long("review-type")
                 .help("Scrape 'all', 'positive' only, or 'negative' only reviews.")
                 .takes_value(true)
        )
        .arg(
            Arg::with_name("resume")
            .short("r")
            .long("resume")
            .help("Resume a scrape rather than starting a new one. A file containing the previous scrape must be provided.")
            .takes_value(false)
            .required_unless("appid")
        )
        .arg(
            Arg::with_name("end_after_zero")
            .short("e")
            .long("end-after-no-new-data")
            .help("Stop the current scrape if the the last batch consisted of all duplicates. Use with resume.")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("scrape_n")
            .short("n")
            .long("number")
            .help("Scrape only up to N * 100 values.")
            .takes_value(true)
            //.conflicts_with("end_after_zero")
        )
        .arg(Arg::with_name("fail_on_error")
             .short("f")
             .long("fail-resume-parse")
             .help("Fail if parse errors are encountered while resuming a scrape else skip bad data.")
        )
        .arg(Arg::with_name("cache_size")
              .short("s")
              .long("with-cache-size")
              .help("Set a cache size in number of items. Trade off is more disk writes (lower) versus more memory use (higher). Defaults to 500.")
              .takes_value(true)
        )
        .get_matches()
}

fn build_scraper<IterMapFn>(matches: ArgMatches<'static>) -> ScraperAppSettings<IterMapFn>
where
    IterMapFn: EndAfterZero,
{
    let matches = build_arguments();

    // Path to either resume a scrape or where to save a new one.
    // Output paths are required in all uses of my program so we can crash here.
    let path = matches.value_of("OUTPUT").expect("Required output path not found. You need to pass a path to save the scrape's result (or to load a scrape to continue).");
    let review_type =
        matches
            .value_of("review_type")
            .map_or_else(ReviewType::default, |review_type| {
                match review_type.to_lowercase().as_str() {
                    "all" => ReviewType::All,
                    "positive" => ReviewType::Positive,
                    "negative" => ReviewType::Negative,
                    _ => ReviewType::default(),
                }
            });

    // Ending after all duplicate data is optional. Using day_range requires Filter::All which "always" returns data according to the documentation. So, I'm not sure whether this
    // should be mandatory when resuming a scrape (i.e. because day_range and Filter::All are used with a cursor).
    let end_after_zero = matches.is_present("end_after_zero");
    // Whether to fail on an error during parsing a previous scrape.
    let scrape_n = matches.value_of("scrape_n").and_then(|n| {
        // Convert to an Option instead of a Result; panic if negative.
        n.parse::<i32>().ok().and_then(|number| {
            if number.is_positive() {
                Some(number as usize)
            } else {
                panic!(r#""number" must positive (you can't scrape a negative amount)."#);
            }
        })
    });
    let fail_on_error = matches.is_present("fail_on_error");
    // Parse the cache size if any or return a default.
    let cache_size = matches
        .value_of("cache_size")
        .map_or(DEFAULT_CACHE_SIZE, |s| {
            s.parse().unwrap_or(DEFAULT_CACHE_SIZE)
        });

    // Logging useful informational bits
    info!("Using cache size: {}", cache_size);
    warn!(
        "Quitting after receiving all duplicates or after N scrapes? {}",
        if end_after_zero | scrape_n.is_some() {
            "YES."
        } else {
            "NO. You may have to manually close the scraper or else it will run and pull data for a very long time."
        }
    );
    info!("Scraping {} reviews", review_type.as_str());

    if matches.is_present("resume") {
        info!("Resuming a scrape using the file: {}", path);

        // Build the cache by attempting to resume from the path.
        // It's okay to panic here during initialization because the program can't continue if the file loading fails.
        let ResumeScraperCache { cache, resume_info } = match ScraperCache::resume_from_file(cache_size, path, fail_on_error) {
            Ok(resume_scraper_cache) => resume_scraper_cache,
            Err(Error::MultipleAppids) => panic!("The provided file ({}) contains multiple appids. Resuming multiple appids isn't supported.", path),
            Err(Error::Io(e)) => io_error_handler(e.kind(), path),
            Err(e) => panic!("Error while resuming scrape: {}", e)
        };

        // Calculate the number of days to go back based on the timestamps.
        // I assume this works.
        let last_scraped_time = Utc.timestamp(resume_info.timestamp.into(), 0);
        let current_time = offset::Utc::now();
        let days_ago = (current_time - last_scraped_time).num_days();

        if days_ago.is_negative() {
            panic!(
                r#"The earliest timestamp in the provided file is more recent than today; check the provided file again.
                   Last scraped time: {}
                   Current time UTC: {}
                   Elapsed days: {}"#,
                last_scraped_time, current_time, days_ago
            );
        } else {
            let mut review_api = ReviewApi::new(resume_info.appid.as_ref().parse().unwrap());
            review_api.filter(Filter::All)
                .expect("Failed to change the Filter to Filter::All for resuming a scrape. This shouldn't happen.")
                .day_range(days_ago.try_into().expect(&format!("days_ago can't fit into a u32 for some reason: {}", days_ago)))
                .expect("Failed to set day_range while resuming a scrape; this is surely a bug.")
                .num_per_page(100)
                .review_type(review_type);

            let scraper: ReviewScraper = review_api
                .try_into()
                .expect("Error when building a scraper from the Steam API after parsing args.");

            let scraper = wrap_scraper(scraper, scrape_n, end_after_zero);
            ScraperAppSettings { scraper, cache }
        }
    } else {
        let appid = matches
            .value_of("appid")
            .expect("You must pass an appid if you're not resuming a scrape")
            .parse()
            .expect("Improper appid passed. Appids are numbers such as 1235140.");

        let scraper: ReviewScraper = ReviewApi::new(appid)
            .num_per_page(100)
            .review_type(review_type)
            .try_into()
            .expect("Error when building a scraper from the Steam API after parsing args.");

        let cache = match ScraperCache::new(cache_size, path) {
            Ok(cache) => cache,
            Err(Error::Io(e)) => io_error_handler(e.kind(), path),
            Err(e) => panic!("Error when building scraper cache: {}", e),
        };

        info!(
            "Beginning a new scrape of appid {} and writing to {}.",
            appid, path
        );

        let scraper = wrap_scraper(scraper, scrape_n, end_after_zero);
        ScraperAppSettings { scraper, cache }
    }
}
