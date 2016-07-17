use std::str::FromStr;

extern crate sodiumoxide as sodium;
extern crate rustc_serialize;
extern crate rfc1751;

use sodium::crypto::box_;
use sodium::crypto::box_::PublicKey;
use sodium::crypto::box_::SecretKey as PrivateKey;
use sodium::crypto::box_::Nonce;
use sodium::crypto::box_::NONCEBYTES;

use rustc_serialize::hex::{FromHex, ToHex};
use rustc_serialize::base64;
use rustc_serialize::base64::{FromBase64, ToBase64};
use rfc1751::{FromRfc1751, ToRfc1751};

static BASE64_CONFIG: base64::Config = base64::Config {
    char_set: base64::CharacterSet::UrlSafe,
    newline: base64::Newline::LF,
    pad: false,
    line_length: None
};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum PtsError {
    FatalInit,
    FatalEncode,
    PublicKeyParse,
    PublicKeyLength,
    PrivateKeyParse,
    PrivateKeyLength,
    DecryptParse,
    DecryptPhase,
    DecryptLength,
}
use PtsError as PE;

struct PublicKeyWrapper(PublicKey);
struct PrivateKeyWrapper(PrivateKey);

impl FromStr for PublicKeyWrapper {
    type Err = PtsError;
    fn from_str(s: &str) -> Result<PublicKeyWrapper, PtsError> {
        let s_mod = s.trim().to_uppercase();
        let material = try!(s_mod.from_rfc1751().or(Err(PE::PublicKeyParse)));
        let key = try!(PublicKey::from_slice(&material).ok_or(PE::PublicKeyLength));
        Ok(PublicKeyWrapper(key))
    }
}

impl FromStr for PrivateKeyWrapper {
    type Err = PtsError;
    fn from_str(s: &str) -> Result<PrivateKeyWrapper, PtsError> {
        let s_mod = s.trim().to_lowercase();
        let material = try!(s_mod.from_hex().or(Err(PE::PrivateKeyParse)));
        let key = try!(PrivateKey::from_slice(&material).ok_or(PE::PrivateKeyLength));
        Ok(PrivateKeyWrapper(key))
    }
}

pub fn init() -> Result<(), PtsError> {
    if sodium::init() {
        Ok(())
    } else {
        Err(PE::FatalInit)
    }
}

pub fn new_keypair() -> Result<(String, String), PtsError> {
    let (PublicKey(public_key), PrivateKey(private_key)) = box_::gen_keypair();
    let public_key_rep = try!(public_key.to_rfc1751().or(Err(PE::FatalEncode)));
    let private_key_rep = private_key.to_hex();
    Ok((public_key_rep.to_lowercase(), private_key_rep))
}

pub fn encrypt(public_key_str: &str, private_key_str: &str, message_bytes: &[u8])
    -> Result<String, PtsError> {
    let PublicKeyWrapper(public_key) = try!(public_key_str.parse());
    let PrivateKeyWrapper(private_key) = try!(private_key_str.parse());
    let nonce = box_::gen_nonce();
    let Nonce(nonce_bytes) = nonce;
    let mut cipher_bytes = box_::seal(message_bytes, &nonce, &public_key, &private_key);
    let mut output_bytes = cipher_bytes;
    output_bytes.extend_from_slice(&nonce_bytes);
    Ok(output_bytes.to_base64(BASE64_CONFIG))
}

pub fn decrypt(public_key_str: &str, private_key_str: &str, cipher_text: &str)
    -> Result<Vec<u8>, PtsError> {
    let PublicKeyWrapper(public_key) = try!(public_key_str.parse());
    let PrivateKeyWrapper(private_key) = try!(private_key_str.parse());
    let input_bytes = try!(cipher_text.from_base64().or(Err(PE::DecryptParse)));
    let len = input_bytes.len();
    if len > box_::NONCEBYTES {
        let nonce_bytes = &input_bytes[len-NONCEBYTES..];
        let nonce = try!(Nonce::from_slice(nonce_bytes).ok_or(PE::DecryptParse));
        let cipher_bytes = &input_bytes[..len-NONCEBYTES];
        let plain_bytes = try!(box_::open(cipher_bytes, &nonce, &public_key, &private_key)
                               .or(Err(PE::DecryptPhase)));
        Ok(plain_bytes)
    } else {
        Err(PE::DecryptLength)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
