// use anyhow::{Context, Result};
use csv::Writer;
use hash_hasher::{HashBuildHasher, HashedSet};
use review_scraper::ReviewScraper;
use std::{
    collections::hash_map::DefaultHasher,
    //convert::TryInto,
    fs::File,
    hash::{Hash, Hasher},
    io,
    iter::FromIterator,
    path::Path,
};
use steam_review_api::{convenience_structs::flat_query::FlattenedQuery, RevApiError, ReviewApi};

pub struct ScraperCache {
    // Hashes of FlattenedQuery.
    // The set only stores hashes by using hash_hasher in order to keep memory use down.
    seen_set: HashedSet<u64>,
    // Cache of FlattenedQuery that is kept from growing/reallocating.
    cache: Vec<FlattenedQuery>,
    // Current index of unwritten data
    write_index: usize,
    // Internal scraper used to pull queries.
    scraper: ReviewScraper,
    // CSV file.
    file: Writer<File>,
}

impl ScraperCache {
    pub fn new<P>(scraper: ReviewScraper, cache_size: usize, path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        // Write to a new file rather than resuming a scrape.
        let csv_file = File::with_options()
            .create_new(true)
            .write(true)
            .open(path)?;
        let csv_writer = Writer::from_writer(csv_file);

        Ok(Self {
            seen_set: HashedSet::with_capacity_and_hasher(cache_size, HashBuildHasher::default()),
            cache: Vec::with_capacity(cache_size),
            write_index: 0,
            scraper,
            file: csv_writer,
        })
    }

    /*#[inline]
    pub fn new_from_api<P>(api: ReviewApi, cache_size: usize, path: P) -> Result<Self, RevApiError>
    where
        P: AsRef<Path>,
    {
        ScraperCache::new(api.try_into()?, cache_size, path)
    }*/

    /// Write the entire cache out to file or resume a failed write.
    pub fn flush_cache(&mut self) -> Result<(), csv::Error> {
        // I'm not draining the cache in order to handle errors if necessary.
        // Draining would clear the cache once the iterator is dropped.
        for (i, query) in self.cache.iter().enumerate().skip(self.write_index) {
            if let Err(e) = self.file.serialize(query) {
                self.write_index = i;
                return Err(e);
            }
        }

        self.cache.clear();
        self.write_index = 0;
        Ok(())
    }

    /// Amount of free space left in the internal cache.
    #[inline]
    pub fn cache_free_space(&self) -> usize {
        self.cache.capacity() - self.cache.len()
    }

    /// Is the cache full?
    #[inline]
    pub fn cache_full(&self) -> bool {
        self.cache.capacity() == self.cache.len()
    }

    // Filter the latest set of queries by checking if the hash exists in seen_set.
    fn filter_data<'a, B>(&self, new_data: &'a [FlattenedQuery]) -> B
    where
        B: FromIterator<(&'a FlattenedQuery, u64)>,
    {
        new_data
            .iter()
            .filter_map(|flat| {
                // According to the documentation, a new Hasher is needed per hash.
                // https://doc.rust-lang.org/std/hash/trait.Hasher.html#tymethod.finish
                let mut hasher = DefaultHasher::new();
                flat.hash(&mut hasher);
                let hash = hasher.finish();

                // Filter out existing hashes since that means we wrote/processed them already.
                if self.seen_set.contains(&hash) {
                    None
                } else {
                    Some((flat, hash))
                }
            })
            // AFAIK, I can't return an impl Iterator here because of &self's lifetime.
            // However, returning an impl Iterator would be ideal rather than creating a new collection.
            .collect()
    }

    // Updates the scraper's cache without triggering reallocations.
    // The cache should stay at or below the size provided during construction.
    fn update_cache(&mut self, item: &FlattenedQuery) -> Result<(), csv::Error> {
        if self.cache_full() {
            self.flush_cache()?;
            debug_assert!(self.cache.len() < self.cache.capacity());
        }

        self.cache.push(item.clone());
        Ok(())
    }

    // Add the filtered queries and hashes to the internal caches.
    fn process_data(&mut self, data: &[(&FlattenedQuery, u64)]) -> Result<(), csv::Error> {
        for (flat, hash) in data {
            self.seen_set.insert(*hash);
            self.update_cache(flat)?;
        }

        Ok(())
    }

    pub fn insert(&mut self, data: &[FlattenedQuery]) -> Result<(), csv::Error> {
        let filtered_data: Vec<_> = self.filter_data(data);
        self.process_data(&filtered_data)
    }
}

impl Drop for ScraperCache {
    fn drop(&mut self) {
        if let Err(e) = self.flush_cache() {
            eprintln!(
                "Failed to flush remaining cache on quit. Lost: {} rows.",
                self.cache.len()
            );
        }
    }
}

// Don't remember why I was doin' this.
/*
impl TryFrom<(ReviewApi, usize)> for ScraperManager {
    type Error = RevApiError;

    fn try_from(api: (ReviewApi, usize)) -> Result<Self, Self::Error> {
        let scraper: ReviewScraper = api.0.try_into()?;
        Ok(ScraperManager::new(scraper, api.1))
    }
}
*/
