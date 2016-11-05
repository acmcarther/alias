extern crate clap;
extern crate hyper;
extern crate api;
extern crate redis;

mod mapping;
mod server;
mod database;

use api::Command;
use api::InvalidSuffix;
use api::Suffix;
use database::InMemoryClient;
use database::RedisClient;

pub fn main() {
  //server::run(InMemoryClient::new(), "0.0.0.0:7721");
  server::run(RedisClient::new(), "0.0.0.0:7721");
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {}
}
