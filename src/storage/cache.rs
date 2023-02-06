use std::ops::Deref;
use std::sync::Arc;
use std::{collections::HashMap, marker::Send};

use chrono::{DateTime, Duration, NaiveTime, Utc};
use maiq_api_models::polling::{Change, Poll, SnapshotChanges};
use maiq_parser::{fetch_snapshot, utils, Fetch, Snapshot};

use rocket::serde::json::Json;
use tokio::time;
use tokio::{sync::RwLock, time::Interval};

use crate::{api::error::ApiError, env, storage::MongoPool};

use super::SnapshotPool;

pub fn run_updater(cache: Arc<RwLock<CachePool>>) {
  tokio::spawn(async move {
    let cache = cache.clone();
    loop {
      let cache_ref = cache.clone();
      let cleaner = tokio::spawn(async move {
        loop {
          let now = utils::now(0).time();
          let wait_s = NaiveTime::from_hms_opt(23, 59, 59)
            .unwrap()
            .signed_duration_since(now)
            .num_seconds()
            + 1;

          info!("Waiting for {}s to drop previous day poll", wait_s);
          tokio::time::sleep(std::time::Duration::from_secs(wait_s as u64)).await;
          cache_ref.write().await.reset();
        }
      });

      let cache_ref = cache.clone();

      let updater = tokio::spawn(async move {
        let mut cache_interval = get_interval_from_env();
        cache_interval.tick().await;
        loop {
          info!("Sleeping for {:?}", cache_interval.period());
          cache_interval.tick().await;
          cache_ref.write().await.update_tick().await;
        }
      });

      _ = tokio::join!(cleaner, updater);
      error!("Seems snapshot updater is panicked. Restarting thread in 10s!");
      tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
  });
}

fn get_interval_from_env() -> Interval {
  let interval_secs = env::parse_var(env::UPDATE_INTERVAL).unwrap();
  time::interval(std::time::Duration::from_secs(interval_secs))
}

struct CachedSnapshot {
  added: DateTime<Utc>,
  snapshot: Snapshot,
}

impl Deref for CachedSnapshot {
  type Target = Snapshot;

  fn deref(&self) -> &Self::Target {
    &self.snapshot
  }
}

impl CachedSnapshot {
  pub fn since_added(&self) -> Duration {
    utils::now(0) - self.added
  }
}

impl From<Snapshot> for CachedSnapshot {
  fn from(s: Snapshot) -> Self {
    Self { added: utils::now(0), snapshot: s }
  }
}

pub struct CachePool {
  cached: Vec<CachedSnapshot>,
  cached_poll: Json<Poll>,
  poll: Poll,

  pub interval: Interval,
  cache_size: usize,
  cache_age_limit: Duration,

  db: MongoPool,
}

impl CachePool {
  pub async fn new(mongo: MongoPool) -> Arc<RwLock<Self>> {
    let interval = get_interval_from_env();
    let mut pool = Self {
      interval,
      cached: vec![],
      cache_size: env::parse_var(env::CACHE_SIZE).unwrap(),
      cache_age_limit: Duration::seconds(env::parse_var(env::CACHE_AGE_LIMIT).unwrap()),
      poll: Poll::default(),
      cached_poll: Json(Poll::default()),
      db: mongo,
    };

    pool.update_tick().await;

    Arc::new(RwLock::new(pool))
  }

  pub fn poll(&self) -> Json<Poll> {
    self.cached_poll.clone()
  }

  pub fn collect_all(&self) -> Vec<Snapshot> {
    self.cached.iter().map(|s| s.snapshot.clone()).collect()
  }

  pub async fn update_tick(&mut self) {
    info!("Updating cache..");
    self.purge();

    _ = self.update(Fetch::Today).await;
    _ = self.update(Fetch::Next).await;

    self.cache_poll();
    info!("Poll updated to:\n{:?}", self.poll);
  }

  pub fn reset(&mut self) {
    warn!("Poll dropped!");
    self.poll = Poll::default();
  }

  fn cache_poll(&mut self) {
    let filter = |kv: &HashMap<String, Change>| {
      kv.iter()
        .filter(|x| !x.1.is_same())
        .map(|x| (x.0.clone(), x.1.clone()))
        .collect()
    };

    let today = SnapshotChanges { uid: self.poll.today.uid.clone(), groups: filter(&self.poll.today.groups) };
    let next = SnapshotChanges { uid: self.poll.next.uid.clone(), groups: filter(&self.poll.next.groups) };

    self.cached_poll = Json(Poll { today, next, next_update: self.poll.next_update.clone() });
  }

  async fn update(&mut self, fetch: Fetch) -> Result<(), ApiError> {
    let snapshot = fetch_snapshot(fetch.clone()).await.ok();

    info!("Parsed snapshot {}", snapshot.as_ref().and_then(|s| Some(s.uid.as_str())).unwrap_or("None"));
    let next_update = utils::now(0) + chrono::Duration::from_std(self.interval.period()).unwrap() + Duration::seconds(5);
    self
      .poll
      .update(snapshot.as_ref(), fetch.clone(), next_update.clone());

    if let Some(s) = snapshot.as_ref() {
      self.save(&s).await?;
      if self.db.by_uid(&s.uid).await?.is_none() {
        self.db.save(&s).await?;
      }

      return Ok(());
    }

    Ok(())
  }

  fn purge(&mut self) {
    let len = self.cached.len();
    let now = utils::now_date(0);
    if len > self.cache_size {
      self
        .cached
        .retain(|s| s.since_added() < self.cache_age_limit || s.date >= now);
      info!("Removed {} snapshots from cache", len - self.cached.len())
    }
  }
}

#[rocket::async_trait]
impl SnapshotPool for CachePool {
  async fn save(&mut self, snapshot: &Snapshot) -> Result<(), ApiError> {
    if let Some(_) = self.cached.iter_mut().find(|s| s.uid.as_str() == snapshot.uid) {
      return Ok(());
    }

    if let Some(index) = self.cached.iter_mut().position(|s| s.date == snapshot.date) {
      info!("Removing snapshot by date {} due to receiving new", snapshot.date);
      self.cached.remove(index);
    }

    info!("Snapshot {} saved to cache", snapshot.uid);
    self.cached.push(snapshot.clone().into());
    return Ok(());
  }

  async fn latest(&self, mode: Fetch) -> Result<Option<Snapshot>, ApiError> {
    let today = utils::now_date(0);
    let mut iter = self.cached.iter().rev();
    let res = match mode {
      Fetch::Today => iter.find(|s| s.date == today).map(|c| c.snapshot.clone()),
      Fetch::Next => iter.find(|s| s.date > today).map(|c| c.snapshot.clone()),
    };

    Ok(res)
  }

  async fn by_uid<T: AsRef<str> + Send>(&self, uid: T) -> Result<Option<Snapshot>, ApiError> {
    let res = self
      .cached
      .iter()
      .find(|s| s.uid.as_str() == uid.as_ref())
      .map(|c| c.snapshot.clone());

    Ok(res)
  }
}
