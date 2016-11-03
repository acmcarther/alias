use database::DatabaseClient;
use hyper::Url;
use hyper::header::Location;
use hyper::method::Method;
use hyper::server::Handler;
use hyper::server::Request;
use hyper::server::Response;
use hyper::server::Server;
use hyper::status::StatusCode;
use hyper::uri::RequestUri;
use mapping::Mapping;
use std::io::Read;
use std::mem;
use std::net::ToSocketAddrs;

pub fn run<D: DatabaseClient + 'static, To: ToSocketAddrs>(db: D, addr: To) {
  Server::http(addr)
    .unwrap()
    .handle(AliasHandler::new(db))
    .unwrap();
}

struct AliasHandler<D: DatabaseClient + 'static> {
  db_client: D,
}

impl<D: DatabaseClient + 'static> AliasHandler<D> {
  pub fn new(client: D) -> AliasHandler<D> {
    AliasHandler { db_client: client }
  }

  pub fn set(&self, body: Vec<u8>, res: Response) {
    let msg = String::from_utf8(body).unwrap();

    let parts: Vec<&str> = msg.split_whitespace().collect();

    if parts.len() != 2 {
      res.send(b"Expected \"$(uri) $(url)\"");
      return;
    }

    let m: &str = parts.get(0).unwrap();
    let mapping_part = m.to_owned();
    let mapping = Mapping::Custom(mapping_part);
    match Url::parse(parts.get(1).unwrap()) {
      Ok(url) => {
        if self.db_client.set_mapping(mapping, url) {
          res.send(b"I did it").unwrap()
        } else {
          res.send(b"Thats already taken").unwrap()
        }
      },
      Err(_) => res.send(b"Url did not parse").unwrap(),
    };
  }

  pub fn must_post(&self, res: Response) {
    res.send(b"must post here").unwrap();
  }

  pub fn must_get(&self, res: Response) {
    res.send(b"must get here").unwrap();
  }

  pub fn get(&self, binding: &str, mut res: Response) {
    if let Some(url) = self.db_client.get_mapping(&Mapping::Custom(binding.to_owned())) {
      res.headers_mut().set(Location(url.to_string()));
      mem::swap(res.status_mut(), &mut StatusCode::TemporaryRedirect);

      res.send(&Vec::new()).unwrap()
    } else {
      res.send(b"I dont know that binding");
    }
  }

  pub fn not_handled(&self, res: Response) {
    res.send(b"that is not handled").unwrap();
  }
}

impl<D: DatabaseClient + 'static> Handler for AliasHandler<D> {
  fn handle(&self, mut req: Request, res: Response) {
    let mut body = Vec::new();
    req.read_to_end(&mut body);

    if let RequestUri::AbsolutePath(ref path) = req.uri {
      match (&req.method, path.as_str()) {
        (&Method::Post, "/api/set") => self.set(body, res),
        (&Method::Get, "/api/set") => self.must_post(res),
        (&Method::Get, binding) => self.get(binding, res),
        _ => self.must_get(res),
      };
    } else {
      self.not_handled(res)
    };

  }
}
