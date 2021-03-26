#![feature(entry_insert)]
mod steam_review_api;
use steam_review_api::{buildapi::ReviewApi, reviewtype::ReviewType};

fn main() {
    // Yakuza: Like a Dragon's appid.
    // I'll turn this into a test later.
    let mut steam = ReviewApi::new(1235140);
    steam.review_type(ReviewType::All);
    let built_api = steam.build().unwrap();

    println!("{}", built_api);
}
