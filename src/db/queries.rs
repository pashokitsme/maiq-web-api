use chrono::{Days, Duration, Utc};
use maiq_parser::timetable::Snapshot;
use mongodb::{
  bson::{doc, Bson},
  options::FindOptions,
  Collection,
};

use super::{MongoError, MongoPool};

pub async fn save(mongo: &MongoPool, snapshot: &Snapshot) -> Result<Option<Bson>, MongoError> {
  let snapshots = snapshots(&mongo);
  let mut cur = snapshots.find(doc! { "uid": &snapshot.uid }, None).await?;

  if cur.advance().await? == true {
    info!("That snapshot #{} already exists", snapshot.uid);
    return Ok(None);
  }

  info!("Saving new snapshot #{}", snapshot.uid);
  let res = snapshots.insert_one(snapshot, None).await?;
  Ok(Some(res.inserted_id))
}

pub async fn get_latest_today(mongo: &MongoPool) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = snapshots(&mongo);
  let today = date_timestamp(0);
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
  let snapshots = snapshots(&mongo);
  let time = date_timestamp(1);
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
  let snapshots = snapshots(&mongo);
  let mut cur = snapshots.find(doc! { "uid": uid }, None).await?;
  if !cur.advance().await? {
    warn!("Snapshot #{} not found", uid);
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

fn snapshots(mongo: &MongoPool) -> Collection<Snapshot> {
  mongo.default_database().unwrap().collection("snapshots")
}

fn date_timestamp(offset: u64) -> i64 {
  let now = Utc::now().naive_utc() + Duration::seconds(60 * 60 * 3);
  info!("{}", now);
  now
    .date()
    .checked_add_days(Days::new(offset))
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .timestamp()
}
