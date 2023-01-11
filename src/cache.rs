use std::ops::Deref;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use maiq_parser::{fetch_n_parse, utils, Fetch, Snapshot};
use serde::Serialize;
use tokio::{
  sync::Mutex,
  time::{self, Interval},
};

use crate::{api::error::ApiError, db::MongoPool, env};

#[derive(Serialize, Clone, Debug)]
pub struct Poll {
  pub latest_today_uid: Option<String>,
  pub latest_next_uid: Option<String>,
  pub last_update: DateTime<Utc>,
  pub next_update: DateTime<Utc>,
}

#[derive(Debug, Default)]
struct InnerPoll {
  pub latest_today_uid: Option<String>,
  pub latest_next_uid: Option<String>,
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
  poll: InnerPoll,

  pub interval: Interval,
  cache_size: usize,
  cache_age_limit: Duration,

  db: MongoPool,
}

impl CachePool {
  pub async fn new(mongo: MongoPool) -> Arc<Mutex<Self>> {
    let interval = get_interval_from_env();
    let mut pool = Self {
      last_update: utils::now(0),
      next_update: utils::now(0) + Duration::seconds(interval.period().as_secs() as i64),
      interval,
      cached: vec![],
      cache_size: env::parse_var(env::CACHE_SIZE).unwrap(),
      cache_age_limit: Duration::seconds(env::parse_var(env::CACHE_AGE_LIMIT).unwrap()),
      poll: InnerPoll::default(),
      db: mongo,
    };

    pool.update_tick().await;

    Arc::new(Mutex::new(pool))
  }

  pub fn cached<'a>(&mut self, mode: Fetch) -> Option<Snapshot> {
    let today = utils::now_date(0);
    let mut iter = self.cached.iter_mut().rev();
    match mode {
      Fetch::Today => iter.find(|s| s.date == today).map(cached_to_snapshot),
      Fetch::Tomorrow => iter.find(|s| s.date > today).map(cached_to_snapshot),
    }
  }

  pub fn cached_by_uid<'a, 'b>(&mut self, uid: &'a str) -> Option<Snapshot> {
    self
      .cached
      .iter_mut()
      .find(|s| s.uid.as_str() == uid)
      .map(cached_to_snapshot)
  }

  pub fn poll(&self) -> Poll {
    Poll {
      latest_today_uid: self.poll.latest_today_uid.clone(),
      latest_next_uid: self.poll.latest_next_uid.clone(),
      last_update: self.last_update,
      next_update: self.next_update,
    }
  }

  pub fn collect_all(&self) -> Vec<Snapshot> {
    self.cached.iter().map(|s| s.snapshot.clone()).collect()
  }

  pub async fn update_tick(&mut self) {
    info!("Updating cache..");
    self.purge();

    self.last_update = utils::now(0);

    let today = self.update(Fetch::Today).await;
    let next = self.update(Fetch::Tomorrow).await;
    possible_error_handler(today, next);
    self.next_update = utils::now(0) + chrono::Duration::from_std(self.interval.period()).unwrap();
  }

  async fn update(&mut self, fetch: Fetch) -> Result<(), ApiError> {
    let snapshot = fetch_n_parse(&fetch).await?.snapshot;
    info!("Parsed snapshot {}", snapshot.uid);
    let latest = self.db.get_by_uid(snapshot.uid.as_str()).await?;

    match fetch {
      Fetch::Today => {
        self.poll.latest_today_uid = match snapshot.date {
          x if x == utils::now(0) => Some(snapshot.uid.clone()),
          _ => None,
        }
      }
      Fetch::Tomorrow => self.poll.latest_next_uid = Some(snapshot.uid.clone()),
    }

    self.try_cache_snapshot(&snapshot);
    debug!("Set poll: {:?}", &self.poll);

    if latest.is_none() {
      debug!("Saving snapshot..");
      self.db.save(snapshot).await?;
    }

    Ok(())
  }

  pub fn try_cache_snapshot(&mut self, snapshot: &Snapshot) -> bool {
    if let Some(_) = self.cached.iter_mut().find(|s| s.uid.as_str() == snapshot.uid) {
      return false;
    }

    info!("Caching snapshot {}", snapshot.uid);
    self.cached.push(snapshot.clone().into());
    return true;
  }

  fn purge(&mut self) {
    let len = self.cached.len();
    if len > self.cache_size {
      self.cached.retain(|s| s.since_added() < self.cache_age_limit);
      info!("Removed {} snapshots from cache", len - self.cached.len())
    }
  }
}

pub fn get_interval_from_env() -> Interval {
  let interval_secs = env::parse_var(env::UPDATE_INTERVAL).unwrap();
  time::interval(std::time::Duration::from_secs(interval_secs))
}

fn possible_error_handler(today: Result<(), ApiError>, next: Result<(), ApiError>) {
  if let Err(err) = today {
    error!("Error while updating cache for today: {}", err);
  }

  if let Err(err) = next {
    error!("Error while updating cache for next day: {}", err);
  }
}

fn cached_to_snapshot(cached: &mut CachedSnapshot) -> Snapshot {
  cached.added = utils::now(0);
  cached.snapshot.clone()
}
