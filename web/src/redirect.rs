use database::RedisClient;
use iron::Handler;
use iron::IronError;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::Url;
use iron::modifiers::Redirect;
use iron::status;
use mapping::Mapping;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Clone)]
pub struct RedirectConfig {
  client: RedisClient,
}

impl RedirectConfig {
  pub fn new(c: RedisClient) -> RedirectConfig {
    RedirectConfig { client: c }
  }

  pub fn find_mapping(&self, route: &str) -> Option<Url> {
    self.client.get_mapping(&Mapping::Custom(route.to_owned()))
  }

  pub fn install_mapping(&self, route: &str, url: Url) -> bool {
    self.client.set_mapping(Mapping::Custom(route.to_owned()), url)
  }

  pub fn list_all(&self) -> Vec<String> {
    self.client
      .list_all()
      .into_iter()
      .map(|m| match m {
        Mapping::Generated(cs) => cs.iter().cloned().collect(),
        Mapping::Custom(s) => s.to_owned(),
      })
      .collect()
  }
}

pub struct RedirecterInner {
  pub config: RedirectConfig,
}

impl RedirecterInner {
  pub fn new(config: RedirectConfig) -> RedirecterInner {
    RedirecterInner { config: config }
  }
}

pub struct Redirecter {
  inner: Arc<RedirecterInner>,
}

#[derive(Debug)]
pub struct NoMapping;

impl fmt::Display for NoMapping {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("No matching mapping")
  }
}

impl Error for NoMapping {
  fn description(&self) -> &str {
    "No mapping"
  }
}

fn perform_redirect(url: Url) -> Response {
  Response::new()
    .set(Redirect(url))
    .set(status::TemporaryRedirect)
}

impl Redirecter {
  pub fn new(config: RedirectConfig) -> Redirecter {
    Redirecter { inner: Arc::new(RedirecterInner::new(config)) }
  }

  fn handle_method(&self, req: &mut Request, path: &str) -> IronResult<Response> {
    self.inner
      .config
      .find_mapping(path)
      .map(|url| perform_redirect(url))
      .ok_or(IronError::new(NoMapping, status::NotFound))
  }
}

impl Handler for Redirecter {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    let path = req.url.path().join("/");
    self.handle_method(req, &path)
  }
}
