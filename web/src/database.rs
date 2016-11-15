use iron::Url;
use mapping::Mapping;
use redis::Commands;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

fn build_key(prefix: &str, m: &Mapping) -> String {
  format!("{}::{}", prefix, map_to_string(m))
}

fn map_to_string(m: &Mapping) -> String {
  match m {
    &Mapping::Generated(ref cs) => cs.iter().cloned().collect(),
    &Mapping::Custom(ref s) => s.clone(),
  }
}

#[derive(Clone)]
pub struct RedisClient {
  key_prefix: String,
  store: String,
}

impl RedisClient {
  pub fn new() -> RedisClient {
    RedisClient {
      store: "redis://redis.service.consul".to_owned(),
      key_prefix: "alias::test::1".to_owned(),
    }
  }

  pub fn with_suffix(&self, suffix: &str) -> RedisClient {
    RedisClient {
      store: self.store.clone(),
      key_prefix: format!("{}{}", self.key_prefix, suffix),
    }
  }

  pub fn list_all(&self) -> Vec<Mapping> {
    let conn = ::redis::Client::open(self.store.as_str()).and_then(|c| c.get_connection()).unwrap();

    let keys: Vec<String> = conn.keys(format!("{}::*", self.key_prefix)).unwrap();
    keys.into_iter().map(|k| Mapping::Custom(k)).collect()
  }

  pub fn set_mapping(&self, m: Mapping, url: Url) -> bool {
    let conn = ::redis::Client::open(self.store.as_str()).and_then(|c| c.get_connection()).unwrap();
    // SETNX -- write if key doesn't exist
    let res: u32 = conn.set_nx(build_key(self.key_prefix.as_str(), &m), url.to_string()).unwrap();

    // SETNX returns 1 if we wrote
    res == 1
  }

  pub fn get_mapping(&self, m: &Mapping) -> Option<Url> {
    let conn = ::redis::Client::open(self.store.as_str()).and_then(|c| c.get_connection()).unwrap();
    // TODO(acmcarther): Add caching at this level
    let res: Option<String> = conn.get(build_key(self.key_prefix.as_str(), m)).unwrap();
    res.and_then(|s| Url::parse(&s).ok())
  }

  pub fn drop_mapping(&self, m: &Mapping) -> bool {
    let conn = ::redis::Client::open(self.store.as_str()).and_then(|c| c.get_connection()).unwrap();
    let res: u32 = conn.del(build_key(self.key_prefix.as_str(), m)).unwrap();

    // DEL returns the number of values deleted
    res != 0
  }

  pub fn has_mapping(&self, m: &Mapping) -> bool {
    let conn = ::redis::Client::open(self.store.as_str()).and_then(|c| c.get_connection()).unwrap();
    let res: u32 = conn.exists(build_key(self.key_prefix.as_str(), m)).unwrap();

    // EXISTS returns 1 if exists, 0 if not
    res != 0
  }
}
