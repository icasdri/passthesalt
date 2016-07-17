use std::str::FromStr;

extern crate sodiumoxide as sodium;
extern crate rustc_serialize;
extern crate rfc1751;

use sodium::crypto::box_;
use sodium::crypto::box_::PublicKey;
use sodium::crypto::box_::SecretKey as PrivateKey;

use rustc_serialize::hex::{FromHex, ToHex};
use rustc_serialize::base64::{FromBase64, ToBase64};
use rustc_serialize::base64::Config as Base64Config;
use rfc1751::{FromRfc1751, ToRfc1751};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum PtsError {
    FatalInit,
    FatalEncode,
    PublicKeyParse,
    PublicKeyLength,
    PrivateKeyParse,
    PrivateKeyLength,
}
use PtsError as PE;

struct PublicKeyWrapper(PublicKey);
struct PrivateKeyWrapper(PrivateKey);

impl FromStr for PublicKeyWrapper {
    type Err = PtsError;
    fn from_str(s: &str) -> Result<PublicKeyWrapper, PtsError> {
        let s_mod = s.trim().to_uppercase();
        let material = try!(s_mod.from_rfc1751()
                            .map_err(|_| PE::PublicKeyParse));
        let key = try!(PublicKey::from_slice(&material)
                       .ok_or(PE::PublicKeyLength));
        Ok(PublicKeyWrapper(key))
    }
}

impl FromStr for PrivateKeyWrapper {
    type Err = PtsError;
    fn from_str(s: &str) -> Result<PrivateKeyWrapper, PtsError> {
        let s_mod = s.trim().to_lowercase();
        let material = try!(s_mod.from_hex()
                            .map_err(|_| PE::PrivateKeyParse));
        let key = try!(PrivateKey::from_slice(&material)
                       .ok_or(PE::PrivateKeyLength));
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
    let public_key_rep = try!(public_key.to_rfc1751()
        .map_err(|_| PE::FatalEncode))
        .to_lowercase();
    let private_key_rep = private_key.to_hex();
    Ok((public_key_rep, private_key_rep))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
