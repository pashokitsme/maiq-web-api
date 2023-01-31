use std::collections::HashMap;
use std::marker::Send;
use std::ops::Deref;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use maiq_parser::{fetch_snapshot, utils, Fetch, Snapshot};
use serde::Serialize;
use tokio::{
  sync::RwLock,
  time::{self, Interval},
};

use crate::{api::error::ApiError, env, storage::MongoPool};

use super::SnapshotPool;

#[derive(Serialize, Clone, Debug, Default)]
pub struct Poll {
  pub today: Option<InnerPoll>,
  pub next: Option<InnerPoll>,
  pub last_update: DateTime<Utc>,
  pub next_update: DateTime<Utc>,
}

#[derive(Serialize, Default, Debug, Clone)]
pub struct InnerPoll {
  pub uid: String,
  pub groups: HashMap<String, String>,
}

impl InnerPoll {
  fn from_snapshot(snapshot: Option<&Snapshot>) -> Option<Self> {
    let snapshot = snapshot?;
    let uid = snapshot.uid.clone();
    let groups = snapshot
      .groups
      .iter()
      .map(|g| (g.name.clone(), g.uid.clone()))
      .collect::<HashMap<String, String>>();
    Some(Self { uid, groups })
  }
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
  last_update: DateTime<Utc>,
  next_update: DateTime<Utc>,

  cached: Vec<CachedSnapshot>,
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
      last_update: utils::now(0),
      next_update: utils::now(0) + Duration::seconds(interval.period().as_secs() as i64),
      interval,
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

    self.last_update = utils::now(0);
    self.poll.last_update = self.last_update;

    _ = self.update(Fetch::Today).await;
    _ = self.update(Fetch::Next).await;

    self.next_update = utils::now(0) + chrono::Duration::from_std(self.interval.period()).unwrap() + Duration::seconds(5);
    self.poll.next_update = self.next_update;

    info!("Set poll: {:?}", &self.poll);
  }

  async fn update(&mut self, fetch: Fetch) -> Result<(), ApiError> {
    let snapshot = fetch_snapshot(fetch.clone()).await.ok();

    info!("Got snapshot: {}", snapshot.as_ref().and_then(|s| Some(s.uid.as_str())).unwrap_or("None"));

    match fetch {
      Fetch::Today => self.poll.today = InnerPoll::from_snapshot(snapshot.as_ref()),
      Fetch::Next => self.poll.next = InnerPoll::from_snapshot(snapshot.as_ref()),
    }

    if let Some(s) = snapshot.as_ref() {
      self.save(&s).await?;
      if self.db.by_uid(&s.uid).await?.is_none() {
        debug!("Saving snapshot..");
        self.db.save(&s).await?;
      }

      return Ok(());
    }

    if let Some(s) = &self.db.latest(fetch).await? {
      self.save(s).await?;
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

    info!("Cached: {}", snapshot.uid);
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

pub fn get_interval_from_env() -> Interval {
  let interval_secs = env::parse_var(env::UPDATE_INTERVAL).unwrap();
  time::interval(std::time::Duration::from_secs(interval_secs))
}
