#[cfg(feature = "comparing")]
use log::info;
#[cfg(feature = "comparing")]
use maiq_shared::{Fetch, Snapshot};
#[cfg(feature = "comparing")]
use std::env;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Poll {
  pub today: SnapshotChanges,
  pub next: SnapshotChanges,
  pub next_update: DateTime<Utc>,
}

#[cfg(feature = "comparing")]
impl Poll {
  pub fn update(&mut self, snapshot: Option<&Snapshot>, fetch: Fetch, next_update: DateTime<Utc>) {
    self.next_update = next_update;

    if snapshot.is_none() {
      return match fetch {
        Fetch::Today => self.today = SnapshotChanges::default(),
        Fetch::Next => self.next = SnapshotChanges::default(),
      };
    }

    let snapshot = snapshot.as_ref().unwrap();

    match fetch {
      Fetch::Today => self.today = self.today.distinct(snapshot),
      Fetch::Next => self.next = self.next.distinct(snapshot),
    }
  }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SnapshotChanges {
  pub uid: Option<String>,
  pub groups: HashMap<String, Change>,
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
#[serde(tag = "change", content = "uid")]
pub enum Change {
  Update(Option<String>),
  New(String),
  Same(Option<String>),
  #[default]
  None,
}

impl Change {
  pub fn uid(&self) -> Option<&str> {
    match self {
      Change::Same(Some(x)) | Change::New(x) | Change::Update(Some(x)) => Some(x),
      Change::None | Change::Same(None) | Change::Update(None) => None,
    }
  }

  pub fn is_same(&self) -> bool {
    match self {
      Self::Same(_) => true,
      _ => false,
    }
  }

  pub fn is_same_with(&self, other: &str) -> bool {
    match self.uid() {
      Some(x) => x == other,
      _ => false,
    }
  }

  pub fn is_not_same_with(&self, other: &str) -> bool {
    match self.uid() {
      Some(x) => x != other,
      _ => false,
    }
  }
}

#[cfg(feature = "comparing")]
impl SnapshotChanges {
  pub fn distinct(&self, snapshot: &Snapshot) -> Self {
    info!("Comparing changes {:?} with snapshot {:?}", self.uid, snapshot.uid);
    let mut changes = Self::default_groups_map();

    for (group_name, group_change) in changes.iter_mut() {
      let prev = self
        .groups
        .iter()
        .find(|g| *g.0 == *group_name)
        .and_then(|x| Some(x.1));

      let new = snapshot
        .groups
        .iter()
        .find(|g| g.name == *group_name)
        .map(|g| (&*g.name, &*g.uid));

      *group_change = match (prev, new) {
        (Some(prev), None) => match prev {
          Change::None | Change::Update(None) => Change::Same(None),
          Change::Same(Some(_)) | Change::New(_) | Change::Update(Some(_)) => Change::Update(None),
          rest => rest.clone(),
        },
        (Some(prev), Some(new)) if prev.is_same_with(&*new.1) => Change::Same(Some(new.1.to_string())),
        (Some(prev), Some(new)) if prev.is_not_same_with(&*new.1) => Change::Update(Some(new.1.to_string())),
        (None, Some(new)) => Change::New(new.1.to_string()),
        _ => continue,
      };
    }

    Self { uid: Some(snapshot.uid.clone()), groups: changes }
  }

  fn default_groups_map() -> HashMap<String, Change> {
    match env::var("GROUPS") {
      Ok(g) => g.split(' ').map(|g| (g.to_owned(), Change::None)).collect(),
      Err(_) => HashMap::new(),
    }
  }
}
