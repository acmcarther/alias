extern crate clap;
extern crate hyper;
extern crate redis;
extern crate iron;
extern crate mount;
extern crate router;
extern crate params;
extern crate rand;
extern crate plugin;
extern crate staticfile;

mod database;
mod list;
mod set;
mod server;
mod mapping;
mod redirect;

use database::RedisClient;

pub fn main() {
  server::run(RedisClient::new(), "0.0.0.0:7721");
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {}
}
