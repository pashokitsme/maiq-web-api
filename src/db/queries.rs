use maiq_parser::timetable::Snapshot;
use mongodb::{bson::doc, options::FindOptions};

use crate::utils;

use super::{get_snapshots, MongoError, MongoPool};

pub async fn get_latest_today(mongo: &MongoPool) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = get_snapshots(&mongo);
  let today = utils::date_timestamp(0);
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots.find(doc! { "date": today }, opts).await?;
  if !cur.advance().await? {
    warn!("There is no snapshots for today");
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

pub async fn get_latest_next(mongo: &MongoPool) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = get_snapshots(&mongo);
  let time = utils::date_timestamp(1);
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots.find(doc! { "date": { "$gte": time } }, opts).await?;
  if !cur.advance().await? {
    warn!("There is no snapshots for next day");
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

pub async fn get_by_uid<'a>(mongo: &MongoPool, uid: &'a str) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = get_snapshots(&mongo);
  let mut cur = snapshots.find(doc! { "uid": uid }, None).await?;
  if !cur.advance().await? {
    warn!("Snapshot #{} not found", uid);
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}
