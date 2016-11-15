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

    println!("provided path: {:?}", path);
    println!("provided url: {:?}", url);

    match (path, url) {
      (Some(&Value::String(ref path)), _) if path_is_invalid(path) => self.path_invalid(),
      (Some(&Value::String(ref path)), Some(&Value::String(ref url))) if path == "" => self.generate_and_install(url),
      (Some(&Value::String(ref path)), Some(&Value::String(ref url))) => self.install(path, url),
      (Some(&Value::Null), Some(&Value::String(ref url))) => self.generate_and_install(url),
      (None, Some(&Value::String(ref url))) => self.generate_and_install(url),
      _ => self.invalid_request(),
    }
  }

  fn install(&self, path: &str, url: &str) -> IronResult<Response> {
    match Url::parse(url) {
      Err(_) => self.invalid_url(),
      Ok(url) => {
        if self.inner.custom_config.install_mapping(path, url) {
          self.success(path)
        } else {
          self.already_exists()
        }
      }
    }
  }

  fn path_invalid(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("Path characters must be alphanumeric (or '-')")
      .set(status::BadRequest))
  }

  fn generate_and_install(&self, url: &str) -> IronResult<Response> {
    match Url::parse(url) {
      Err(_) => self.invalid_url(),
      Ok(url) => {
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
          .set(format!("Set to https://wai.fi/g/{}", gen_url.unwrap()))
          .set(status::Ok))
      }
    }
  }

  fn invalid_request(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("Must POST with 'url', and may contain 'path'")
      .set(status::BadRequest))
  }

  fn success(&self, path: &str) -> IronResult<Response> {
    Ok(Response::new()
      .set(format!("Mapping set to https://wai.fi/c/{}", path))
      .set(status::Ok))
  }

  fn already_exists(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("This mapping already exists (possibly as something else).")
      .set(status::Ok))
  }

  fn invalid_url(&self) -> IronResult<Response> {
    Ok(Response::new()
      .set("Invalid URL, please remember to provide http or https (or other).")
      .set(status::BadRequest))
  }
}

fn path_is_invalid(path: &str) -> bool {
  path.to_owned().chars().any(|c| !c.is_alphanumeric() && c != '-')
}

impl Handler for SetMapping {
  fn handle(&self, req: &mut Request) -> IronResult<Response> {
    self.handle_method(req)
  }
}
