use base64::prelude::*;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::Rng;
use sha2::Sha512;

fn sha512_pbkdf2(input: &str, iterations: u32, salt: &[u8]) -> String {
    let mut dk = [0u8; 64];
    pbkdf2::<Hmac<Sha512>>(input.as_bytes(), salt, iterations, &mut dk).unwrap();

    let hash = BASE64_STANDARD.encode(dk);
    let salt = BASE64_STANDARD.encode(salt);
    format!("$7${iterations}${salt}${hash}")
}

/// hash a password just like `mosquitto_password` "sha512-pbkdf2" (the default)
pub fn hash_password(pw: &str) -> String {
    let salt: [u8; 12] = rand::rng().random();

    sha512_pbkdf2(pw, 101, &salt)
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parts: Vec<&str> = hash.split('$').collect();

    if parts[1] != "7" {
        unimplemented!("support for other hash types than pbkdf2-sha512");
    }
    if parts.len() != 5 {
        return false;
    }

    let Ok(iterations) = parts[2].parse::<u32>() else {
        return false;
    };

    let Ok(salt) = BASE64_STANDARD.decode(parts[3]) else {
        return false;
    };

    let check = sha512_pbkdf2(password, iterations, &salt);
    check == hash
}

#[cfg(test)]
mod tests {
    use base64::prelude::*;

    #[test]
    fn test_hash_password() {
        let pw = "secret";
        // sample generated with mosquitto_passwd
        let sample = "$7$101$du8Zrrg/ivu3ssYA$P82HzKwTXO63NZatuqu5E5gx381rs0Wnj4n3KA8dn5wwgqWGme/M//C4XfPCC1ZebPm2+A4CAleI7BnwqLawiA==";
        let salt = BASE64_STANDARD.decode("du8Zrrg/ivu3ssYA").unwrap();

        let actual = super::sha512_pbkdf2(pw, 101, &salt);
        assert_eq!(actual, sample);
    }

    #[test]
    fn test_verify_password() {
        let pw = "secret";
        let hash = "$7$101$du8Zrrg/ivu3ssYA$P82HzKwTXO63NZatuqu5E5gx381rs0Wnj4n3KA8dn5wwgqWGme/M//C4XfPCC1ZebPm2+A4CAleI7BnwqLawiA==";

        assert!(super::verify_password(hash, pw));
        assert!(!super::verify_password(hash, "wrong"));
    }
}
