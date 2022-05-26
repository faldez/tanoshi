use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use anyhow::anyhow;
use bytes::Bytes;
use fancy_regex::Regex;
use std::convert::TryFrom;

// create an alias for convenience
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

pub enum ImageUri {
    Remote(String),
    File(String),
    Archive(String, String),
}

impl TryFrom<String> for ImageUri {
    type Error = anyhow::Error;

    fn try_from(uri: String) -> Result<Self, Self::Error> {
        let uri = if uri.starts_with("http") {
            Self::Remote(uri)
        } else if !uri.is_empty() {
            let path = std::path::PathBuf::from(&uri);
            if path.is_file() {
                Self::File(uri)
            } else {
                let re = Regex::new(r#"\.(cbz|cbr)[\/|\\]"#)?;

                if let Some(matches) = re.find(&uri)? {
                    let archive = uri[0..matches.end() - 1].to_owned();
                    let filename = uri[matches.end()..uri.len()].to_owned();

                    Self::Archive(archive, filename)
                } else {
                    return Err(anyhow!("invalid file uri"));
                }
            }
        } else {
            return Err(anyhow!("bad uri"));
        };

        Ok(uri)
    }
}

impl ImageUri {
    pub fn from_encrypted(secret: &str, encrypted: &str) -> Result<Self, anyhow::Error> {
        let mut decoded = base64::decode_config(encrypted, base64::URL_SAFE_NO_PAD)?;
        trace!("decoded: {:?}", decoded);

        let iv = [0_u8; 16];

        let bytes = Aes128CbcDec::new(secret.as_bytes().into(), &iv.into())
            .decrypt_padded_mut::<Pkcs7>(&mut decoded)
            .map_err(|e| anyhow::anyhow!("error decrypt url {e}"))?
            .to_vec();

        let url = String::from_utf8(bytes)?;
        let uri = if url.starts_with("http") {
            Self::Remote(url)
        } else if !url.is_empty() {
            let path = std::path::PathBuf::from(&url);
            if path.is_file() {
                Self::File(url)
            } else {
                let re = Regex::new(r#"\.(cbz|cbr)[\/|\\]"#)?;

                if let Some(matches) = re.find(&url)? {
                    let archive = url[0..matches.end() - 1].to_owned();
                    let filename = url[matches.end()..url.len()].to_owned();

                    Self::Archive(archive, filename)
                } else {
                    return Err(anyhow!("invalid file url"));
                }
            }
        } else {
            return Err(anyhow!("bad url"));
        };

        Ok(uri)
    }

    pub fn into_encrypted(self, secret: &str) -> Result<String, anyhow::Error> {
        let uri = self.to_string();
        let pos = uri.len();

        let mut buffer = vec![0_u8; pos * 2];
        buffer.splice(..pos, uri.as_bytes().to_vec());

        let iv = [0_u8; 16];
        let chipertext = Aes128CbcEnc::new(secret.as_bytes().into(), &iv.into())
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
            .map_err(|e| anyhow!("error encrypt url {e}"))?;

        let encoded = base64::encode_config(chipertext, base64::URL_SAFE_NO_PAD);

        Ok(encoded)
    }
}

impl ToString for ImageUri {
    fn to_string(&self) -> String {
        match self {
            ImageUri::Remote(url) => url.to_owned(),
            ImageUri::File(path) => path.to_owned(),
            ImageUri::Archive(archive, filename) => format!("{archive}/{filename}"),
        }
    }
}

pub struct Image {
    pub content_type: String,
    pub data: Bytes,
}
