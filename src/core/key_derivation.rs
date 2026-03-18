use argon2::password_hash::rand_core::OsRng;
use argon2::Argon2;
use rand::RngCore;
use zeroize::Zeroize;

pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| {
            Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
        })?;
    let result = key;
    key.zeroize();
    Ok(result)
}

pub fn is_key_too_weak(key: &[u8; 32]) -> bool {
    use std::collections::HashSet;

    let unique_bytes = key.iter().collect::<HashSet<_>>().len();
    if unique_bytes < 8 {
        return true;
    }

    if key.iter().all(|&b| b == key[0]) {
        return true;
    }

    let is_sequential_asc = key
        .iter()
        .enumerate()
        .all(|(i, &b)| b as i16 == (key[0] as i16 + i as i16) % 256);
    let is_sequential_desc = key
        .iter()
        .enumerate()
        .all(|(i, &b)| b as i16 == (key[0] as i16 - i as i16).wrapping_rem(256));
    if is_sequential_asc || is_sequential_desc {
        return true;
    }

    if key.len() >= 4 {
        let alternating_two = key[0] == key[2] && key[1] == key[3] && key[0] != key[1];
        let all_alternating = key
            .windows(2)
            .all(|w| w[0] == key[0] && w[1] == key[1] || w[0] == key[1] && w[1] == key[0]);
        if alternating_two && all_alternating {
            return true;
        }
    }

    let zeros = key.iter().filter(|&&b| b == 0).count();
    let ones = key.iter().filter(|&&b| b == 0xFF).count();
    if zeros > 24 || ones > 24 {
        return true;
    }

    false
}

pub fn validate_key_strength(key: &[u8; 32]) -> Result<(), &'static str> {
    if is_key_too_weak(key) {
        return Err("Encryption key is too weak (insufficient entropy). Use a strong, random key.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_ne!(salt1, salt2);
        assert_eq!(salt1.len(), 16);
    }

    #[test]
    fn test_derive_key() {
        let password = "test_password";
        let salt = [0u8; 16];
        let key = derive_key(password, &salt);
        assert!(key.is_ok());
        assert_eq!(key.unwrap().len(), 32);
    }

    #[test]
    fn test_is_key_too_weak_sequential() {
        let key: [u8; 32] = (0..32)
            .map(|i| i as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert!(is_key_too_weak(&key));
    }

    #[test]
    fn test_is_key_too_weak_all_zeros() {
        let key = [0u8; 32];
        assert!(is_key_too_weak(&key));
    }

    #[test]
    fn test_is_key_too_weak_max_bytes() {
        let key = [0xFFu8; 32];
        assert!(is_key_too_weak(&key));
    }

    #[test]
    fn test_validate_key_strength() {
        let weak_key = [0u8; 32];
        assert!(validate_key_strength(&weak_key).is_err());

        let strong_key: [u8; 32] = (0..32)
            .map(|i| (i * 7) as u8)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert!(validate_key_strength(&strong_key).is_ok());
    }
}
