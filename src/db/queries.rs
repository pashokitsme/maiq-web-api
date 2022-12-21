use chrono::{Days, Utc};
use maiq_parser::timetable::Snapshot;
use mongodb::{
  bson::{doc, Bson},
  options::FindOptions,
  Collection,
};

use super::{MongoClient, MongoError};

pub async fn save(mongo: &MongoClient, snapshot: &Snapshot) -> Result<Option<Bson>, MongoError> {
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

pub async fn get_latest_today(mongo: &MongoClient) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = snapshots(&mongo);
  let today = current_date();
  info!("{}", today);
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots.find(doc! { "date": today }, opts).await?;
  if cur.advance().await? == false {
    warn!("There is no snapshots for today");
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

pub async fn get_latest_next(mongo: &MongoClient) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = snapshots(&mongo);
  let today = next_date();
  info!("{}", today);
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots
    .find(
      doc! {
            "date": {
               "$gte": today
            }
      },
      opts,
    )
    .await?;
  if !cur.advance().await? {
    warn!("There is no snapshots for next day");
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

pub async fn get_by_uid<'a>(mongo: &MongoClient, uid: &'a str) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = snapshots(&mongo);
  let mut cur = snapshots.find(doc! { "uid": uid }, None).await?;
  if !cur.advance().await? {
    warn!("Snapshot #{} not found", uid);
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

fn snapshots(mongo: &MongoClient) -> Collection<Snapshot> {
  mongo.default_database().unwrap().collection("snapshots")
}

fn current_date() -> i64 {
  Utc::now()
    .date_naive()
    .checked_add_days(Days::new(1))
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .timestamp()
}

fn next_date() -> i64 {
  Utc::now()
    .date_naive()
    .checked_add_days(Days::new(2))
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .timestamp()
}
