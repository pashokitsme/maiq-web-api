use maiq_parser::Snapshot;
use mongodb::bson::{doc, Bson};

use crate::db::SnapshotModel;

use super::{MongoError, MongoPool};

impl MongoPool {
  pub async fn save(&self, snapshot: Snapshot) -> Result<Option<Bson>, MongoError> {
    let snapshots = self.get_snapshot_models();
    let mut cur = snapshots.find(doc! { "uid": &snapshot.uid }, None).await?;

    if cur.advance().await? == true {
      info!("Snapshot {} already exists", snapshot.uid);
      return Ok(None);
    }

    info!("Saving new snapshot {}", snapshot.uid);
    let snapshot_internal = SnapshotModel::from(snapshot);
    let res = snapshots.insert_one(snapshot_internal, None).await?;
    Ok(Some(res.inserted_id))
  }
}
