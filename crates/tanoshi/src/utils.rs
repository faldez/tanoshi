use std::iter;

use aes::Aes128;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use rand::distributions::Alphanumeric;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

// create an alias for convenience
type Aes128Cbc = Cbc<Aes128, Pkcs7>;

#[allow(dead_code)]
fn generate_iv() -> String {
    let mut rng: StdRng = SeedableRng::from_entropy();
    let chars = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(16)
        .collect();
    String::from_utf8(chars).unwrap()
}

pub fn encrypt_url(key: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let pos = url.len();
    let mut buffer = vec![0_u8; pos * 2];
    buffer.splice(..pos, url.as_bytes().to_vec());

    let iv = [0_u8; 16];
    let chiper = Aes128Cbc::new_from_slices(key.as_bytes(), &iv)?;
    let chipertext = chiper.encrypt(&mut buffer, pos)?;

    let encoded = base64::encode_config(chipertext, base64::URL_SAFE_NO_PAD);

    Ok(encoded)
}

pub fn decrypt_url(key: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut decoded = base64::decode_config(data, base64::URL_SAFE_NO_PAD)?;
    debug!("decoded: {:?}", decoded);

    let iv = [0_u8; 16];

    let chiper = Aes128Cbc::new_from_slices(key.as_bytes(), &iv)?;
    let bytes = chiper.decrypt(&mut decoded)?.to_vec();

    let url = String::from_utf8(bytes)?;
    Ok(url)
}

/*
use jsonwebtoken::{self, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub url: String,
    pub exp: i64,
}

pub fn sign_url(secret: &str, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let token = jsonwebtoken::encode(
        &Header::default(),
        &Claims {
            url: url.to_string(),
            exp: 3600,
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn validate_url(secret: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let token = jsonwebtoken::decode::<Claims>(
        data,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token.claims.url)
}

*/

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encrypt_url() {
        let key = "pdn8QwMUTDSVfKQf".to_string();
        let url = "https://official-ongoing-2.gamindustri.us/manga/Jujutsu-Kaisen/0006-001.png"
            .to_string();

        let result = encrypt_url(&key, &url);

        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_url() {
        let key = "pdn8QwMUTDSVfKQf".to_string();
        let url = "iSNS4boMrEewCKHEZ-qD6VvrgH4kU92mg-9AlQXcLWHi4LEmNpavXbsHAwIXGQDLwGlS4HNPuyiHNBCECS0S7JQyW8Iz4L_7AQbKARYtThQ".to_string();

        let result = decrypt_url(&key, &url);

        assert!(result.is_ok());
        if let Ok(url) = result {
            assert_eq!(
                "https://official-ongoing-2.gamindustri.us/manga/Jujutsu-Kaisen/0006-001.png",
                url
            );
        }
    }
}
