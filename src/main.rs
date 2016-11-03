extern crate clap;
extern crate hyper;
extern crate api;

mod mapping;
mod server;
mod database;

use api::Command;
use api::InvalidSuffix;
use api::Suffix;
use database::InMemoryClient;

pub fn main() {
  server::run(InMemoryClient::new(), "0.0.0.0:9286");
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {}
}
