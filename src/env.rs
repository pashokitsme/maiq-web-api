use std::str::FromStr;

macro_rules! env_var {
  ($var_name: ident, $env_name: literal) => {
    pub const $var_name: &'static str = $env_name;
  };
  ($var_name: ident) => {
    pub const $var_name: &'static str = stringify!($var_name);
  };
}

env_var!(API_SECRET);

env_var!(UPDATE_INTERVAL, "UPDATE_CACHE_INTERVAL_SECS");
env_var!(CACHE_SIZE);
env_var!(CACHE_AGE_LIMIT, "CACHE_AGE_LIMIT_SECS");

env_var!(DB_URL, "DATABASE_CONNECTION_URL");
env_var!(DEFAULT_DB, "DEFAULT_DATABASE_NAME");

pub fn parse_var<T: FromStr>(var: &'static str) -> Option<T> {
  self::var(var).and_then(|x| x.parse().ok())
}

pub fn var(var: &'static str) -> Option<String> {
  dotenvy::var(var).ok()
}

pub fn check<T: FromStr>(var: &'static str) -> bool {
  match parse_var::<T>(var) {
    Some(_) => true,
    None => {
      error!("Var {}: {} is not present", var, std::any::type_name::<T>().split("::").last().unwrap());
      false
    }
  }
}

pub fn check_env_vars() {
  info!("Validating .env vars");
  let mut failed = false;

  failed |= !check::<String>(API_SECRET);

  failed |= !check::<u64>(UPDATE_INTERVAL);
  failed |= !check::<usize>(CACHE_SIZE);
  failed |= !check::<u64>(CACHE_AGE_LIMIT);

  failed |= !check::<String>(DB_URL);
  failed |= !check::<String>(DEFAULT_DB);

  failed.then(|| panic!("Not all environment args are set"));
}
