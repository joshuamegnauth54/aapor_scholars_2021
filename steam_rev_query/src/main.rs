use review_scraper::ReviewScraper;
use std::convert::TryInto;
use steam_review_api::{Filter, ReviewApi, ReviewType};

use std::collections::HashSet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let mut steam = ReviewApi::new(1235140);
    let mut steam = ReviewApi::new(57925);
    steam
        .review_type(ReviewType::All)
        .filter(Filter::Recent)
        .unwrap()
        .num_per_page(100);

    let mut scraper: ReviewScraper = steam.try_into()?;
    //println!("{}", built_api);
    //let to_send = get(built_api);

    //let resps: SteamRevOuter = resp.json()?;
    //println!("{:?}", resps);
    let test: Vec<_> = scraper.pull()?;
    println!("First pull: {:?}", test);
    println!("Second pull: ");
    let test_two: HashSet<_> = scraper.pull()?;
    println!("Data: {:?}", test_two);
    Ok(())
}
