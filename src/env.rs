use std::str::FromStr;

use chrono::Duration;
use lazy_static::lazy_static;
use std::ops::Deref;

macro_rules! env_params {
  {$($inner: ty as $tt: ident { $closure: expr } ),*} => {
    $(
      #[derive(Debug, Clone, Copy)]
      pub struct $tt($inner);

      impl Deref for $tt {
        type Target = $inner;
        fn deref(&self) -> &Self::Target { &self.0 }
      }

      impl FromStr for $tt {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
          Ok($tt($closure(s).map_err(|_| ())?))
        }
      }
    )*
  };
}

macro_rules! env_default {
  {$($tt: ty => $default: expr),*} => {
    $(
      impl Default for $tt {
        fn default() -> Self { Self($default) }
      }
    )*
  }
}

macro_rules! vars {
  {$($getter: ident ($var_name: ident) -> $ty: tt),*} => {
    lazy_static! {
      $(static ref $var_name: $ty = self::parse_var::<$ty>(stringify!($var_name));)*
    }

    $(pub fn $getter() -> $ty { $var_name.clone() })*

    pub fn init() {
      $(
        self::var(stringify!($var_name))
          .and_then(|x| x.parse::<$ty>().ok())
          .is_none()
          .then(|| warn!("Value {} of type {} is missing. Fallback to default", stringify!($var_name), stringify!($ty)));
      )*
    }
  };
}

pub fn var(var: &'static str) -> Option<String> {
  dotenvy::var(var).ok()
}

pub fn parse_var<T: FromStr + Default>(var: &'static str) -> T {
  self::var(var).and_then(|x| x.parse().ok()).unwrap_or_default()
}

env_params! {
  Duration as Secs { |s: &str| s.parse().map(Duration::seconds) }
}

env_default! {
  Secs => Duration::seconds(10)
}

vars! {
  update_rate (UPDATE_CACHE_INTERVAL_SECS) -> u64,
  cache_size (CACHE_SIZE) -> usize,
  cache_age_limit (CACHE_AGE_LIMIT_SECS) -> Secs,
  db_url (DATABASE_CONNECTION_URL) -> String,
  db_default_collection (DEFAULT_DATABASE_NAME) -> String,
  api_secret (API_SECRET) -> String
}
