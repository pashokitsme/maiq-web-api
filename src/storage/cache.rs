use std::marker::Send;
use std::ops::Deref;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use maiq_api_wrapper::Poll;
use maiq_parser::compare::distinct;
use maiq_parser::{fetch_snapshot, utils::time::*, Fetch, Snapshot};

use tokio::time;
use tokio::{sync::RwLock, time::Interval};

use crate::{api::error::ApiError, env, storage::MongoPool};

use super::SnapshotPool;

pub fn interval() -> Interval {
  time::interval(std::time::Duration::from_secs(env::parse_var(env::UPDATE_INTERVAL).unwrap()))
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
    now() - self.added
  }
}

impl From<Snapshot> for CachedSnapshot {
  fn from(s: Snapshot) -> Self {
    Self { added: now(), snapshot: s }
  }
}

pub struct CachePool {
  cached: Vec<CachedSnapshot>,
  poll: Poll,
  interval: Interval,
  cache_size: usize,
  cache_age_limit: Duration,
  db: MongoPool,
}

impl CachePool {
  pub async fn new(mongo: MongoPool) -> Arc<RwLock<Self>> {
    let mut pool = Self {
      interval: interval(),
      cached: vec![],
      cache_size: env::parse_var(env::CACHE_SIZE).unwrap(),
      cache_age_limit: Duration::seconds(env::parse_var(env::CACHE_AGE_LIMIT).unwrap()),
      poll: Poll::default(),
      db: mongo,
    };

    pool.update_tick().await;
    Arc::new(RwLock::new(pool))
  }

  pub fn poll(&self) -> Poll {
    self.poll.clone()
  }

  pub fn collect_all(&self) -> Vec<Snapshot> {
    self.cached.iter().map(|s| s.snapshot.clone()).collect()
  }

  pub async fn update_tick(&mut self) {
    info!("Updating cache..");
    self.purge();

    _ = self.update(Fetch::Today).await;
    _ = self.update(Fetch::Next).await;

    let next_update = now() + chrono::Duration::from_std(self.interval.period()).unwrap() + Duration::seconds(5);
    self.poll.next_update = next_update;
    info!("Poll updated has been updated to:");
    info!("Today: {:?}", self.poll.today_changes);
    info!("Next: {:?}", self.poll.next_changes);
  }

  pub fn reset(&mut self) {
    self.poll = Poll::default();
  }

  async fn update(&mut self, fetch: Fetch) -> Result<(), ApiError> {
    let snapshot = fetch_snapshot(&fetch).await.ok();

    info!("Parsed snapshot {}", snapshot.as_ref().map(|s| s.uid.as_str()).unwrap_or("-"));
    if let Some(s) = snapshot.as_ref() {
      self.save(s).await?;
      if self.db.by_uid(&s.uid).await?.is_none() {
        self.db.save(s).await?;
      }
    }

    match fetch {
      Fetch::Today => self.poll.today_changes = distinct(self.poll.today_snapshot.as_ref(), snapshot.as_ref()),
      Fetch::Next => self.poll.next_changes = distinct(self.poll.next_snapshot.as_ref(), snapshot.as_ref()),
    }

    match fetch {
      Fetch::Today => self.poll.today_snapshot = snapshot,
      Fetch::Next => self.poll.next_snapshot = snapshot,
    }

    Ok(())
  }

  fn purge(&mut self) {
    let len = self.cached.len();
    let now = now_date();
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
    if self.cached.iter().any(|s| *s.uid.as_str() == snapshot.uid) {
      return Ok(());
    }

    if let Some(index) = self.cached.iter().position(|s| s.date == snapshot.date) {
      info!("Removing snapshot by date {} due to receiving new", snapshot.date);
      self.cached.remove(index);
    }

    info!("Snapshot {} saved to cache", snapshot.uid);
    self.cached.push(snapshot.clone().into());
    return Ok(());
  }

  async fn latest(&self, mode: Fetch) -> Result<Option<Snapshot>, ApiError> {
    let today = now_date();
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
