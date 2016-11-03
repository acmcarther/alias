use hyper::Url;
use mapping::Mapping;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub trait DatabaseClient: Send + Sync {
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
