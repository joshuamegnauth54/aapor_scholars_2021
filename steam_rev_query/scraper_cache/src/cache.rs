use csv::{Reader, Writer};
use hash_hasher::{HashBuildHasher, HashedSet};
use rev_query_utils::{
    error::{Error, Result},
    resumeinfo::ResumeInfo,
};
use std::{
    collections::hash_map::DefaultHasher,
    fs::File,
    hash::{Hash, Hasher},
    iter::FromIterator,
    path::Path,
};
use steam_review_api::convenience_structs::flat_query::FlattenedQuery;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct ResumeScraperCache {
    pub cache: ScraperCache,
    pub resume_info: ResumeInfo,
}

#[derive(Debug)]
pub struct ScraperCache {
    // Hashes of FlattenedQuery.
    // The set only stores hashes by using hash_hasher in order to keep memory use down.
    seen_set: HashedSet<u64>,
    // Cache of FlattenedQuery that is kept from growing/reallocating.
    cache: Vec<FlattenedQuery>,
    // Current index of unwritten data
    write_index: usize,
    // CSV file.
    file: Writer<File>,
}

impl ScraperCache {
    pub fn new<P>(cache_size: usize, path: P) -> Result<Self>
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
            file: csv_writer,
        })
    }

    /// Resume a scrape from a CSV file.
    /// Scrapes are only resumeable from a single appid. The file specified by `path` shouldn't contain
    /// multiple appids. Timestamps are required.
    #[tracing::instrument]
    pub fn resume_from_file<P>(
        cache_size: usize,
        path: P,
        // Fail on errors while parsing if true else skip the row.
        fail_on_error: bool,
    ) -> Result<ResumeScraperCache>
    where
        P: AsRef<Path> + std::fmt::Debug + std::fmt::Display,
    {
        let mut seen_set =
            HashedSet::with_capacity_and_hasher(cache_size, HashBuildHasher::default());
        let mut resume_info = ResumeInfo::default();

        {
            let mut csv_reader = Reader::from_path(&path)?;
            for flat_query in csv_reader.deserialize::<FlattenedQuery>() {
                match flat_query {
                    Ok(flat_query) => {
                        let mut hasher = DefaultHasher::new();
                        flat_query.hash(&mut hasher);
                        let hash = hasher.finish();
                        seen_set.insert(hash);

                        resume_info.update(&flat_query)?;
                    }
                    Err(e) if fail_on_error => return Err(e.into()),
                    Err(e) => {
                        error!(
                            "WARNING: Failed to parse a row of the CSV: {}.\nError given: {}",
                            path, e
                        )
                    }
                }
            }
        }

        // Append to a scrape
        let csv_file = File::with_options().append(true).open(path)?;
        let csv_writer = Writer::from_writer(csv_file);

        Ok(ResumeScraperCache {
            cache: ScraperCache {
                seen_set,
                cache: Vec::with_capacity(cache_size),
                write_index: 0,
                file: csv_writer,
            },
            resume_info,
        })
    }

    /// Write the entire cache out to file or resume a failed write.
    pub fn flush_cache(&mut self) -> Result<()> {
        // I'm not draining the cache in order to handle errors if necessary.
        // Draining would clear the cache once the iterator is dropped.
        for (i, query) in self.cache.iter().enumerate().skip(self.write_index) {
            if let Err(e) = self.file.serialize(query) {
                self.write_index = i;
                warn!(
                    "Write index set to {} because serializing a query failed.",
                    self.write_index
                );
                return Err(e.into());
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
    fn filter_data<'iter, B>(&self, new_data: &'iter [FlattenedQuery]) -> B
    where
        B: FromIterator<(&'iter FlattenedQuery, u64)>,
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
    fn update_cache(&mut self, item: &FlattenedQuery) -> Result<()> {
        if self.cache_full() {
            self.flush_cache()?;
            debug_assert!(self.cache.len() < self.cache.capacity());
        }

        self.cache.push(item.clone());
        Ok(())
    }

    // Add the filtered queries and hashes to the internal caches.
    fn process_data(&mut self, data: &[(&FlattenedQuery, u64)]) -> Result<()> {
        for (flat, hash) in data {
            self.seen_set.insert(*hash);
            self.update_cache(flat)?;
        }

        Ok(())
    }

    #[tracing::instrument]
    pub fn insert(&mut self, data: &[FlattenedQuery]) -> Result<()> {
        let filtered_data: Vec<_> = self.filter_data(data);
        let length = filtered_data.len();
        if length > 0 {
            info!("{} valid, unique nodes scraped.", length);
            self.process_data(&filtered_data)
        } else {
            warn!("Scraped all duplicate nodes or zero nodes total.");
            Err(Error::NoDataAfterFiltering)
        }
    }
}

// Ensure that the cached data are written out when the cache is dropped.
impl Drop for ScraperCache {
    #[tracing::instrument]
    fn drop(&mut self) {
        if let Err(e) = self.flush_cache() {
            error!(
                "Failed to flush remaining cache on quit. Lost: {} rows.",
                self.cache.len()
            );
            warn!("Error on cache flush: {}", e);
        }
    }
}
