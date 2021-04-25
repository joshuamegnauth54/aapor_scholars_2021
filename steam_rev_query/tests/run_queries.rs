use review_scraper::ReviewScraper;
use std::convert::TryInto;
use steam_review_api::ReviewApi;

// Several App IDs that are useful for testing. The list below may be overkill.
// I've found that different product types have different optional
// values so I'd like to test all of them.

// Application (Blender).
const APPLICATION: u32 = 365670;
// Left 4 Dead 2's dedicated server.
const SERVER: u32 = 222860;
// Game with many reviews (Dark Souls III).
const GAME_MANY: u32 = 374320;
// DLC (Overcooked! 2 - Carnival of Chaos).
const DLC: u32 = 1138400;
// Game removed from the store (Duke Nukem 3D: Megaton Edition).
const REMOVED: u32 = 225140;
// Anime/video removed from Steam (Love Live movie).
const ANIME: u32 = 773900;
// Community-made mod (Skyrim Script Extender)
const COMM_MOD: u32 = 365720;
// Game with small-ish amount of reviews (Master Levels for Doom 2)
// Used to complete a full scrape.
const LOW_REVS: u32 = 9160;

fn test_base(appid: u32) {
    let mut query: ReviewScraper = ReviewApi::new(appid)
        .try_into()
        .expect("Building basic query failed.");
    let _response = query
        .next()
        .expect("Expected to pull data but our response is empty.")
        .expect("Querying data failed");
}

#[test]
fn test_app() {
    test_base(APPLICATION);
}
#[test]
fn test_server() {
    test_base(SERVER);
}

#[test]
fn test_game_many() {
    test_base(GAME_MANY);
}

#[test]
fn test_dlc() {
    test_base(DLC);
}

#[test]
fn test_removed() {
    test_base(REMOVED);
}

#[test]
fn test_anime() {
    test_base(ANIME);
}

#[test]
fn test_comm_mod() {
    test_base(COMM_MOD);
}

// Query that exhausts all data.
// Tests if the Iterator is properly returning None as well as
// if the scraper behaves well over several queries.
#[test]
fn full_scrape() {
    let scraper: ReviewScraper = ReviewApi::new(LOW_REVS)
        .num_per_page(100)
        .try_into()
        .expect("Building full_scrape query failed");

    for response in scraper {
        let _data = response.expect("Pulling query failed.");
    }
}
