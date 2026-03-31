use std::{
  fmt::{self, Display, Formatter},
  str::FromStr,
};

use rand::RngExt;
use serde::{Deserialize, Serialize};

/// Alphabet used to encode bytes into the `k`-`z` character range.
const REVERSE_HEX_CHARS: &[u8; 16] = b"zyxwvutsrqponmlk";

/// A 128-bit identifier encoded as a 32-character string using the `k`-`z` alphabet.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct Id([u8; 16]);

impl Id {
  pub fn new() -> Self {
    let mut rng = rand::rng();
    let mut bytes = [0u8; 16];
    for b in &mut bytes {
      *b = rng.random();
    }
    Self(bytes)
  }

  /// Returns the first 8 characters of the encoded ID as a short prefix.
  pub fn short(&self) -> String {
    encode_prefix(&self.0)
  }
}

impl Default for Id {
  fn default() -> Self {
    Self::new()
  }
}

impl Display for Id {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(&encode(&self.0))
  }
}

impl From<Id> for String {
  fn from(id: Id) -> Self {
    id.to_string()
  }
}

impl FromStr for Id {
  type Err = String;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    if s.len() != 32 {
      return Err(format!("Id must be exactly 32 characters, got {}", s.len()));
    }
    let bytes = decode(s)?;
    Ok(Self(bytes))
  }
}

impl TryFrom<String> for Id {
  type Error = String;

  fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
    s.parse()
  }
}

fn decode(s: &str) -> Result<[u8; 16], String> {
  let chars: Vec<u8> = s.bytes().collect();
  if chars.len() != 32 {
    return Err(format!("Id must be exactly 32 characters, got {}", chars.len()));
  }

  let mut bytes = [0u8; 16];
  for i in 0..16 {
    let high = nibble_from_char(chars[i * 2])?;
    let low = nibble_from_char(chars[i * 2 + 1])?;
    bytes[i] = (high << 4) | low;
  }
  Ok(bytes)
}

fn encode(bytes: &[u8; 16]) -> String {
  let mut s = String::with_capacity(32);
  for &b in bytes {
    let high = (b >> 4) as usize;
    let low = (b & 0x0F) as usize;
    s.push(REVERSE_HEX_CHARS[high] as char);
    s.push(REVERSE_HEX_CHARS[low] as char);
  }
  s
}

/// Encode only the first 4 bytes into an 8-character string for [`Id::short`].
fn encode_prefix(bytes: &[u8; 16]) -> String {
  let mut s = String::with_capacity(8);
  for &b in &bytes[..4] {
    let high = (b >> 4) as usize;
    let low = (b & 0x0F) as usize;
    s.push(REVERSE_HEX_CHARS[high] as char);
    s.push(REVERSE_HEX_CHARS[low] as char);
  }
  s
}

fn nibble_from_char(c: u8) -> Result<u8, String> {
  match c {
    b'k'..=b'z' => Ok(b'z' - c),
    _ => Err(format!("Id must contain only characters k-z, found '{}'", c as char)),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod from_str {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_parses_valid_id() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      assert_eq!(id.to_string(), "zyxwvutsrqponmlkzyxwvutsrqponmlk");
    }

    #[test]
    fn it_rejects_digits() {
      let result = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz1".parse::<Id>();
      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_out_of_range_chars() {
      let result = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzja".parse::<Id>();
      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_too_long() {
      let result = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".parse::<Id>();
      assert!(result.is_err());
    }

    #[test]
    fn it_rejects_too_short() {
      let result = "zzz".parse::<Id>();
      assert!(result.is_err());
    }
  }

  mod new {
    use super::*;

    #[test]
    fn it_generates_valid_ids() {
      let id = Id::new();
      let s = id.to_string();
      assert_eq!(s.len(), 32);
      assert!(s.chars().all(|c| ('k'..='z').contains(&c)));
    }
  }

  mod display {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_encodes_all_ff_bytes_as_all_k() {
      let id = Id([0xFF; 16]);
      assert_eq!(id.to_string(), "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
    }

    #[test]
    fn it_formats_as_the_encoded_string() {
      let id: Id = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz".parse().unwrap();
      assert_eq!(id.to_string(), "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    }
  }

  mod roundtrip {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_roundtrips_through_string() {
      let id = Id::new();
      let s = id.to_string();
      let parsed: Id = s.parse().unwrap();
      assert_eq!(id, parsed);
    }
  }

  mod short {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_returns_first_eight_characters() {
      let id: Id = "zyxwvutsrqponmlkzyxwvutsrqponmlk".parse().unwrap();
      assert_eq!(id.short(), "zyxwvuts");
    }
  }

  mod encode_decode {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_encodes_ff_bytes_as_k() {
      let bytes = [0xFF; 16];
      assert_eq!(encode(&bytes), "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
    }

    #[test]
    fn it_encodes_zero_bytes_as_z() {
      let bytes = [0u8; 16];
      assert_eq!(encode(&bytes), "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
    }

    #[test]
    fn it_roundtrips_encode_decode() {
      let bytes = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF,
      ];
      let encoded = encode(&bytes);
      let decoded = decode(&encoded).unwrap();
      assert_eq!(bytes, decoded);
    }
  }
}
