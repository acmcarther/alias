#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Mapping {
  Generated([char; 5]),
  Custom(String),
}
