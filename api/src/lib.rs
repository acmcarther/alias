#![feature(proc_macro)]

extern crate hyper;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[derive(Serialize, Deserialize, Debug, PartialEq,Eq)]
pub struct Suffix(String);

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum InvalidSuffix {
  InvalidCharacters(Vec<char>),
  NoCharacters,
}

impl Suffix {
  pub fn new(s: &str) -> Result<Suffix, InvalidSuffix> {
    if s.is_empty() {
      return Err(InvalidSuffix::NoCharacters);
    }

    let invalid_chars = s.chars().filter(is_invalid_char).collect::<Vec<char>>();

    if invalid_chars.is_empty() {
      return Ok(Suffix(s.to_owned()));
    } else {
      return Err(InvalidSuffix::InvalidCharacters(invalid_chars));
    }
  }
}

fn is_invalid_char(c: &char) -> bool {
  return !(c.is_alphanumeric() || *c == '-');
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Command {
  CreateAlias(String, Suffix),
  Shorten(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    assert!(Suffix::new("hello123").is_ok());
    assert!(Suffix::new("hello-123").is_ok());
    assert!(!Suffix::new("hello\\123").is_ok());
  }
}
