use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use maiq_parser::{fetch_n_parse, Fetch};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
  api::error::ApiError,
  db::{self, MongoPool},
  utils,
};

#[derive(Serialize, Debug, Clone)]
pub struct Cache {
  pub last_updated: Option<DateTime<Utc>>,
  pub next_update: Option<DateTime<Utc>>,
  pub latest_today_uid: Option<String>,
  pub latest_next_uid: Option<String>,
}

impl Default for Cache {
  fn default() -> Self {
    Self {
      latest_today_uid: Default::default(),
      latest_next_uid: Default::default(),
      last_updated: Default::default(),
      next_update: Default::default(),
    }
  }
}

pub async fn update<'a>(
  mongo: &MongoPool,
  fetch: Fetch,
  cache: &mut Arc<Mutex<Cache>>,
  interval: &Duration,
) -> Result<(), ApiError> {
  info!("Updating cache for {:?}..", fetch);

  let snapshot = fetch_n_parse(&fetch).await?.snapshot;
  let latest = db::get_by_uid(&mongo, snapshot.uid.as_str()).await?;
  let mut locked_cache = cache.lock().await;
  locked_cache.last_updated = Some(Utc::now());
  locked_cache.next_update = Some(Utc::now() + interval.clone());

  match fetch {
    Fetch::Today => {
      locked_cache.latest_today_uid = match snapshot.date {
        x if x == utils::current_date(0) => Some(snapshot.uid.clone()),
        _ => None,
      }
    }
    Fetch::Tomorrow => locked_cache.latest_next_uid = Some(snapshot.uid.clone()),
  }

  debug!("Set cache: {:?}", &locked_cache);

  if latest.is_none() {
    db::save(&mongo, snapshot).await?;
  }

  Ok(())
}
