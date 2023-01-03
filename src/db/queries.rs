use maiq_parser::Snapshot;
use mongodb::{
  bson::{doc, DateTime},
  options::FindOptions,
};

use crate::utils;

use super::{get_snapshots_as_model, MongoError, MongoPool};

pub async fn get_latest_today(mongo: &MongoPool) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = get_snapshots_as_model(&mongo);
  let today = DateTime::from_chrono(utils::current_date(0));
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots.find(doc! { "date": today }, opts).await?;
  if !cur.advance().await? {
    warn!("There is no snapshots for today");
    return Ok(None);
  };

  Ok(Some(cur.deserialize_current()?.into()))
}

pub async fn get_latest_next(mongo: &MongoPool) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = get_snapshots_as_model(&mongo);
  let time = DateTime::from_chrono(utils::current_date(1));
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots.find(doc! { "date": { "$gte": time } }, opts).await?;
  if !cur.advance().await? {
    warn!("There is no snapshots for next day");
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?.into()))
}

pub async fn get_by_uid<'a>(mongo: &MongoPool, uid: &'a str) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = get_snapshots_as_model(&mongo);
  let mut cur = snapshots.find(doc! { "uid": uid }, None).await?;
  if !cur.advance().await? {
    warn!("Snapshot {} not found", uid);
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?.into()))
}
