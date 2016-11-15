use iron::Handler;
use iron::IronError;
use iron::IronResult;
use iron::Request;
use iron::Response;
use iron::Set;
use iron::Url;
use iron::status;
use params::Params;
use params::Value;
use plugin::Pluggable;
use rand::Rng;
use rand::thread_rng;
use redirect::RedirectConfig;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

pub struct SetMappingInner {
  pub gen_config: RedirectConfig,
  pub custom_config: RedirectConfig,
}

pub struct SetMapping {
  inner: Arc<SetMappingInner>,
}

impl SetMapping {
  pub fn new(gen_config: RedirectConfig, custom_config: RedirectConfig) -> SetMapping {
    SetMapping {
      inner: Arc::new(SetMappingInner {
        gen_config: gen_config,
        custom_config: custom_config,
      }),
    }
  }

  fn handle_method(&self, req: &mut Request) -> IronResult<Response> {
    let map = req.get_ref::<Params>().unwrap();

    let path = map.find(&["path"]);
    let url = map.find(&["url"]);

    match (path, url) {
      (Some(&Value::String(ref path)), Some(&Value::String(ref url))) => self.install(path, url),
      (Some(&Value::Null), Some(&Value::String(ref url))) => self.generate_and_install(url),
      (None, Some(&Value::String(ref url))) => self.generate_and_install(url),
      _ => self.invalid_request(),
    }
  }

  fn install(&self, path: &str, url: &str) -> IronResult<Response> {
    Url::parse(url)
      .map_err(|e| IronError::new(InvalidUrl, status::BadRequest))
      .and_then(|url| {
        if self.inner.custom_config.install_mapping(path, url) {
          self.success()
        } else {
          self.already_exists()
        }
      })
  }

  fn generate_and_install(&self, url: &str) -> IronResult<Response> {
    Url::parse(url)
      .map_err(|e| IronError::new(InvalidUrl, status::BadRequest))
      .and_then(|url| {
        let mut gen_url = None;

        while gen_url.is_none() {
          let chars: String = thread_rng()
            .gen_ascii_chars()
            .filter(|c| {
              let c_32 = *c as u32;
              c_32 >= ('A' as u32) && c_32 <= ('z' as u32)
            })
            .take(5)
            .collect();

          if self.inner.gen_config.install_mapping(chars.as_str(), url.clone()) {
            gen_url = Some(chars)
          }
        }

        Ok(Response::new()
          .set(format!("Set to '/g/{}'", gen_url.unwrap()))
          .set(status::Ok))
      })
  }

  fn invalid_request(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("Must POST with 'url', and may contain 'path'")
      .set(status::BadRequest))
  }

  fn success(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("Mapping set!")
      .set(status::Ok))
  }

  fn already_exists(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("This mapping already exists (possibly as something else).")
      .set(status::Ok))
  }
}

impl Handler for SetMapping {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    self.handle_method(req)
  }
}

#[derive(Debug)]
pub struct InvalidUrl;

impl fmt::Display for InvalidUrl {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("Invalid url: Must match 'https?://url'")
  }
}

impl Error for InvalidUrl {
  fn description(&self) -> &str {
    "Invalid Url"
  }
}
