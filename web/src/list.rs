use iron::Handler;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::status;
use redirect::RedirectConfig;
use std::sync::Arc;

pub struct ListInner {
  pub config: RedirectConfig,
}

pub struct List {
  inner: Arc<ListInner>,
}

impl List {
  pub fn new(config: RedirectConfig) -> List {
    List { inner: Arc::new(ListInner { config: config }) }
  }

  fn handle_method(&self, _: &mut Request) -> IronResult<Response> {
    Ok(build_response(self.inner
      .config
      .list_all()))
  }
}

fn build_response(ms: Vec<String>) -> Response {
  let payload = ms.into_iter().map(|m| format!("{},\n", m)).collect::<String>();

  Response::new()
    .set(payload)
    .set(status::Ok)
}

impl Handler for List {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    self.handle_method(req)
  }
}
