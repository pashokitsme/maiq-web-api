use maiq_parser::timetable::Snapshot;
use mongodb::{
  bson::{doc, Bson},
  options::{ClientOptions, FindOptions},
  Client, Collection,
};

use crate::env;

pub type MongoClient = Client;
pub type MongoError = mongodb::error::Error;

// todo: validating collections on init
pub async fn init() -> Result<MongoClient, MongoError> {
  let url = dotenvy::var(env::DB_URL).unwrap();
  info!("Connecting to {}", url);

  let mut opts = ClientOptions::parse(url).await?;
  opts.app_name = Some("maiq".into());
  opts.default_database = Some("bafoksqiyr3wxpf".into());

  let client = MongoClient::with_options(opts)?;

  Ok(client)
}

pub async fn save(mongo: &MongoClient, snapshot: &Snapshot) -> Result<Option<Bson>, MongoError> {
  let snapshots = snapshots(&mongo);
  let mut search_res = snapshots.find(doc! { "uid": &snapshot.uid }, None).await?;

  if search_res.advance().await? == true {
    info!("That snapshot #{} already exists", snapshot.uid);
    return Ok(None);
  }

  info!("Saving new snapshot #{}", snapshot.uid);
  let res = snapshots.insert_one(snapshot, None).await?;
  Ok(Some(res.inserted_id))
}

pub async fn get_latest_today(mongo: &MongoClient) -> Result<Option<Snapshot>, MongoError> {
  let snapshots = snapshots(&mongo);
  let today = chrono::Utc::now().date_naive().to_string();
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
  let today = chrono::Utc::now();
  let opts = FindOptions::builder()
    .sort(doc! { "parsed_date": 1, "date": 1 })
    .limit(1)
    .build();
  let mut cur = snapshots
    .find(
      doc! {
            "date": {
               "$gt": today
            }
      },
      opts,
    )
    .await?;
  if cur.advance().await? == false {
    warn!("There is no snapshots for next day");
    return Ok(None);
  }

  Ok(Some(cur.deserialize_current()?))
}

fn snapshots(mongo: &MongoClient) -> Collection<Snapshot> {
  mongo.default_database().unwrap().collection("snapshots")
}

// fn date_now() -> NaiveDateTime {
// NaiveDateTime::new(date, time)
// }
