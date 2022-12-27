use maiq_parser::timetable::Snapshot;
use mongodb::bson::{doc, Bson};

use crate::db::get_snapshots;

use super::{MongoError, MongoPool};

pub async fn save(mongo: &MongoPool, snapshot: &Snapshot) -> Result<Option<Bson>, MongoError> {
  let snapshots = get_snapshots(&mongo);
  let mut cur = snapshots.find(doc! { "uid": &snapshot.uid }, None).await?;

  if cur.advance().await? == true {
    info!("Snapshot #{} already exists", snapshot.uid);
    return Ok(None);
  }

  info!("Saving new snapshot #{}", snapshot.uid);
  let res = snapshots.insert_one(snapshot, None).await?;
  Ok(Some(res.inserted_id))
}
