//use steam_review_api::{buildapi::ReviewApi, options::{ReviewType, Filter}};
use steam_review_api::*;

fn main() {
    // Yakuza: Like a Dragon's appid.
    // I'll turn this into a test later.
    let mut steam = ReviewApi::new(1235140);
    steam.review_type(ReviewType::All).filter(Filter::Recent);
    let built_api = steam.build().unwrap();

    println!("{}", built_api);
}
