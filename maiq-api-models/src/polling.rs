use std::{collections::HashMap, env};

use chrono::{DateTime, Utc};
use maiq_shared::{Fetch, Snapshot};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Poll {
  pub today: SnapshotChanges,
  pub next: SnapshotChanges,
  pub next_update: DateTime<Utc>,
}

impl Poll {
  pub fn update(&mut self, snapshot: Option<&Snapshot>, fetch: Fetch, next_update: DateTime<Utc>) {
    self.next_update = next_update;
    if snapshot.is_none() {
      return;
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
  pub uid: String,
  pub groups: HashMap<String, Change>,
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone)]
#[serde(tag = "change", content = "uid")]
pub enum Change {
  Update(String),
  New(String),
  Same(Option<String>),
  #[default]
  None,
}

impl Change {
  pub fn uid(&self) -> Option<&str> {
    match self {
      Change::Same(Some(x)) | Change::New(x) | Change::Update(x) => Some(x),
      Change::None | Change::Same(None) => None,
    }
  }

  pub fn is_same(&self, other: &str) -> bool {
    match self.uid() {
      Some(x) => x == other,
      _ => false,
    }
  }

  pub fn is_not_same(&self, other: &str) -> bool {
    match self.uid() {
      Some(x) => x != other,
      _ => false,
    }
  }
}

impl SnapshotChanges {
  pub fn distinct(&self, snapshot: &Snapshot) -> Self {
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
          Change::None => Change::Same(None),
          rest => rest.clone(),
        },
        (Some(prev), Some(new)) if prev.is_same(&*new.1) => Change::Same(Some(new.1.to_string())),
        (Some(prev), Some(new)) if prev.is_not_same(&*new.1) => Change::Update(new.1.to_string()),
        (None, Some(new)) => Change::New(new.1.to_string()),
        _ => continue,
      };
    }

    Self { uid: snapshot.uid.clone(), groups: changes }
  }

  fn default_groups_map() -> HashMap<String, Change> {
    match env::var("GROUPS") {
      Ok(g) => g.split(' ').map(|g| (g.to_owned(), Change::None)).collect(),
      Err(_) => HashMap::new(),
    }
  }
}
