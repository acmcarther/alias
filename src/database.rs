use hyper::Url;
use mapping::Mapping;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use redis::Commands;

pub trait DatabaseClient: Send + Sync {
  fn list_all(&self) -> Vec<Mapping>;
  fn set_mapping(&self, Mapping, Url) -> bool;
  fn get_mapping(&self, &Mapping) -> Option<Url>;
  fn has_mapping(&self, &Mapping) -> bool;
  fn drop_mapping(&self, &Mapping) -> bool;
}

#[derive(Clone)]
pub struct InMemoryClient {
  store: Arc<Mutex<HashMap<Mapping, Url>>>,
}

impl InMemoryClient {
  pub fn new() -> InMemoryClient {
    InMemoryClient { store: Arc::new(Mutex::new(HashMap::new())) }
  }
}

impl DatabaseClient for InMemoryClient {
  fn set_mapping(&self, m: Mapping, url: Url) -> bool {
    let mut store_guard = self.store.lock().unwrap();
    if !store_guard.contains_key(&m) {
      store_guard.insert(m, url);
      true
    } else {
      false
    }
  }

  fn list_all(&self) -> Vec<Mapping> {
    panic!("Not implemented because I'm lazy");
  }

  fn get_mapping(&self, m: &Mapping) -> Option<Url> {
    self.store.lock().unwrap().get(m).cloned()
  }

  fn drop_mapping(&self, m: &Mapping) -> bool {
    self.store.lock().unwrap().remove(&m).is_some()
  }

  fn has_mapping(&self, m: &Mapping) -> bool {
    self.store.lock().unwrap().contains_key(m)
  }
}

#[derive(Clone, Copy)]
pub struct RedisClient {
  key_prefix: &'static str,
  store: &'static str,
}

impl RedisClient {
  pub fn new() -> RedisClient {
    RedisClient {
      store: "redis://redis.service.consul",
      key_prefix: "alias::test::1"
    }
  }
}

fn build_key(prefix: &'static str, m: &Mapping) -> String {
  format!("{}::{}", prefix, map_to_string(m))
}

fn map_to_string(m: &Mapping) -> String {
  match m {
    &Mapping::Generated(ref cs) => cs.iter().cloned().collect(),
    &Mapping::Custom(ref s) => s.clone()
  }
}

impl DatabaseClient for RedisClient {
  fn list_all(&self) -> Vec<Mapping> {
    let conn = ::redis::Client::open(self.store).and_then(|c| c.get_connection()).unwrap();

    let keys: Vec<String> = conn.keys(format!("{}::*", self.key_prefix)).unwrap();
    keys.into_iter().map(|k| Mapping::Custom(k)).collect()
  }

  fn set_mapping(&self, m: Mapping, url: Url) -> bool {
    let conn = ::redis::Client::open(self.store).and_then(|c| c.get_connection()).unwrap();
    // SETNX -- write if key doesn't exist
    let res: u32 = conn.set_nx(build_key(self.key_prefix, &m), url.to_string()).unwrap();

    // SETNX returns 1 if we wrote
    res == 1
  }

  fn get_mapping(&self, m: &Mapping) -> Option<Url> {
    let conn = ::redis::Client::open(self.store).and_then(|c| c.get_connection()).unwrap();
    // TODO(acmcarther): Add caching at this level
    let res: Option<String> = conn.get(build_key(self.key_prefix, m)).unwrap();
    res.and_then(|s| Url::parse(&s).ok())
  }

  fn drop_mapping(&self, m: &Mapping) -> bool {
    let conn = ::redis::Client::open(self.store).and_then(|c| c.get_connection()).unwrap();
    let res: u32 = conn.del(build_key(self.key_prefix, m)).unwrap();

    // DEL returns the number of values deleted
    res != 0
  }

  fn has_mapping(&self, m: &Mapping) -> bool {
    let conn = ::redis::Client::open(self.store).and_then(|c| c.get_connection()).unwrap();
    let res: u32 = conn.exists(build_key(self.key_prefix, m)).unwrap();

    // EXISTS returns 1 if exists, 0 if not
    res != 0
  }
}
