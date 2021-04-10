use attohttpc::{self, Result};
use steam_review_api::convenience_structs::*;
use steam_review_api::*;

const fn user_agent() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
}

pub fn test() -> Result {
    let mut steam = ReviewApi::new(1235140);
    steam
        .review_type(ReviewType::All)
        .filter(Filter::Recent)
        .unwrap();

    let built_api = steam.build().unwrap();

    println!("{}", built_api);
    // SET HEADERS LATER
    let to_send = attohttpc::get(built_api)
        .try_header_append("User-Agent", user_agent())
        .expect("Oops.");
    let resp = to_send.send()?;
    //println!("{:?}", to_send);

    let resps: SteamRevOuter = resp.json()?;
    println!("{:?}", resps);
    Ok(())
}
