//! Random secrets and how they are stored. Only SHA-256 hashes ever reach the
//! database, so a leaked database cannot be replayed against the API.

use rand::Rng;
use sha2::{Digest, Sha256};

/// Crockford base32 without I, L, O, U: easy to read aloud and retype.
const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A checkpoint stamp the player types into the website, e.g. `STAMP-7KQ2-M9XD`.
pub fn checkpoint_stamp() -> String {
    format!("STAMP-{}-{}", random_chars(4), random_chars(4))
}

/// A long-lived bearer token for Claude Code, e.g. `grd_...`.
pub fn access_token() -> String {
    format!("grd_{}", random_chars(32).to_lowercase())
}

/// An opaque browser session id.
pub fn session_id() -> String {
    random_chars(40)
}

pub fn hash(secret: &str) -> String {
    hex::encode(Sha256::digest(normalize(secret).as_bytes()))
}

/// Stamps are typed by hand: forgive case, spaces and look-alike letters.
pub fn normalize(secret: &str) -> String {
    secret
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c.to_ascii_uppercase() {
            'O' => '0',
            'I' | 'L' => '1',
            other => other,
        })
        .collect()
}

fn random_chars(len: usize) -> String {
    let mut rng = rand::rng();
    (0..len).map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamp_hash_forgives_typing() {
        let stamp = checkpoint_stamp();
        assert_eq!(hash(&stamp), hash(&format!("  {} ", stamp.to_lowercase())));
    }
}
