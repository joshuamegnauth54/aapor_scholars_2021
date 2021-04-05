use attohttpc::{self, Result};
use steam_review_api::convenience_structs::*;
use steam_review_api::*;

pub fn test() -> Result {
    let mut steam = ReviewApi::new(1235140);
    steam
        .review_type(ReviewType::All)
        .filter(Filter::Recent)
        .unwrap();

    let built_api = steam.build().unwrap();

    println!("{}", built_api);
    // SET HEADERS LATER
    let to_send = attohttpc::get(built_api).header_append("user-agent", "Josh scraper test alpha");
    let resp = to_send.send()?;
    //println!("{:?}", to_send);

    let resps: SteamRevOuter = resp.json()?;
    println!("{:?}", resps);
    Ok(())
}
