use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use maiq_parser::{fetch_n_parse, utils, Fetch, Snapshot};
use serde::Serialize;
use tokio::{
  sync::Mutex,
  time::{self, Interval},
};

use crate::{
  api::error::ApiError,
  db::{self, MongoPool},
  env,
};

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

pub struct CachePool {
  last_update: DateTime<Utc>,
  next_update: DateTime<Utc>,

  cached: Vec<Snapshot>,
  poll: InnerPoll,

  pub interval: Interval,
  mongo: MongoPool,
}

impl CachePool {
  pub async fn new(mongo: MongoPool) -> Arc<Mutex<Self>> {
    let interval = get_interval_from_env();
    let mut pool = Self {
      last_update: utils::now(0),
      next_update: utils::now(0) + Duration::seconds(interval.period().as_secs() as i64),
      interval,
      cached: vec![],
      poll: InnerPoll::default(),
      mongo,
    };

    pool.update_tick().await;

    Arc::new(Mutex::new(pool))
  }

  pub async fn cached<'a>(&self, mode: Fetch) -> Option<Snapshot> {
    let today = utils::now_date(0);
    let mut iter = self.cached.iter().rev();
    match mode {
      Fetch::Today => iter.find(|s| s.date == today).cloned(),
      Fetch::Tomorrow => iter.find(|s| s.date > today).cloned(),
    }
  }

  pub async fn cached_by_uid<'a, 'b>(&self, uid: &'a str) -> Option<Snapshot> {
    self.cached.iter().find(|s| s.uid.as_str() == uid).cloned()
  }

  pub fn poll(&self) -> Poll {
    Poll {
      latest_today_uid: self.poll.latest_today_uid.clone(),
      latest_next_uid: self.poll.latest_next_uid.clone(),
      last_update: self.last_update,
      next_update: self.next_update,
    }
  }

  pub async fn update_tick(&mut self) {
    info!("Updating cache..");

    self.last_update = utils::now(0);
    self.next_update = self.last_update + chrono::Duration::from_std(self.interval.period()).unwrap();

    let today = self.update(Fetch::Today).await;
    let next = self.update(Fetch::Tomorrow).await;
    possible_error_handler(today, next);
  }

  async fn update(&mut self, fetch: Fetch) -> Result<(), ApiError> {
    let snapshot = fetch_n_parse(&Fetch::Today).await?.snapshot;
    self.update_cached_snapshots(&snapshot).await;
    let latest = db::get_by_uid(&self.mongo, snapshot.uid.as_str()).await?;

    match fetch {
      Fetch::Today => {
        self.poll.latest_today_uid = match snapshot.date {
          x if x == utils::now(0) => Some(snapshot.uid.clone()),
          _ => None,
        }
      }
      Fetch::Tomorrow => self.poll.latest_next_uid = Some(snapshot.uid.clone()),
    }

    debug!("Set cache: {:?}", &self.poll);

    if latest.is_none() {
      debug!("Saving snapshot..");
      db::save(&self.mongo, snapshot).await?;
    }

    Ok(())
  }

  async fn update_cached_snapshots(&mut self, snapshot: &Snapshot) {
    if !self.is_need_to_update(snapshot).await {
      return;
    }
    self.cached.push(snapshot.clone());
    let len = self.cached.len();
    if len > 4 {
      self.cached.drain(0..(len - 4));
    }
  }

  async fn is_need_to_update(&self, snapshot: &Snapshot) -> bool {
    let cached = self.cached_by_uid(&snapshot.uid).await;
    cached.is_some() || cached.unwrap().age() > Duration::minutes(4)
  }
}

pub fn get_interval_from_env() -> Interval {
  let interval_secs = env::parse_var(env::UPDATE_INTERVAL).unwrap();
  time::interval(std::time::Duration::from_secs(interval_secs))
}

fn possible_error_handler(today: Result<(), ApiError>, next: Result<(), ApiError>) {
  if let Err(err) = today {
    warn!("Error while updating cache for today: {}", err);
  }

  if let Err(err) = next {
    warn!("Error while updating cache for next day: {}", err);
  }
}
