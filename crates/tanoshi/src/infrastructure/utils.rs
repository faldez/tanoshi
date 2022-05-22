use std::iter;

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::distributions::Alphanumeric;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

// create an alias for convenience
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

#[allow(dead_code)]
fn generate_iv() -> String {
    let mut rng: StdRng = SeedableRng::from_entropy();
    let chars = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(16)
        .collect();
    String::from_utf8(chars).unwrap()
}

pub fn encrypt_url(key: &str, url: &str) -> Result<String, anyhow::Error> {
    let pos = url.len();
    let mut buffer = vec![0_u8; pos * 2];
    buffer.splice(..pos, url.as_bytes().to_vec());

    let iv = [0_u8; 16];
    let chipertext = Aes128CbcEnc::new(key.as_bytes().into(), &iv.into())
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
        .map_err(|e| anyhow::anyhow!("error encrypt url {e}"))?;

    let encoded = base64::encode_config(chipertext, base64::URL_SAFE_NO_PAD);

    Ok(encoded)
}

pub fn decrypt_url(key: &str, data: &str) -> Result<String, anyhow::Error> {
    let mut decoded = base64::decode_config(data, base64::URL_SAFE_NO_PAD)?;
    trace!("decoded: {:?}", decoded);

    let iv = [0_u8; 16];

    let bytes = Aes128CbcDec::new(key.as_bytes().into(), &iv.into())
        .decrypt_padded_mut::<Pkcs7>(&mut decoded)
        .map_err(|e| anyhow::anyhow!("error decrypt url {e}"))?
        .to_vec();

    let url = String::from_utf8(bytes)?;
    Ok(url)
}

pub fn decode_cursor(cursor: &str) -> std::result::Result<(i64, i64), base64::DecodeError> {
    match base64::decode(cursor) {
        Ok(res) => {
            let cursor = String::from_utf8(res).unwrap();
            let decoded = cursor
                .split('#')
                .map(|s| s.parse::<i64>().unwrap())
                .collect::<Vec<i64>>();
            Ok((decoded[0], decoded[1]))
        }
        Err(err) => Err(err),
    }
}

pub fn encode_cursor(timestamp: i64, id: i64) -> String {
    base64::encode(format!("{}#{}", timestamp, id))
}

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
