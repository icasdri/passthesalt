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

pub enum PtsError {
    ParseFailed,
    Fatal(&'static str)
}
use PtsError as PE;

pub fn init() -> Result<(), PtsError> {
    if sodium::init() {
        Ok(())
    } else {
        Err(PE::Fatal("Failed to initialize libsodium encryption facilities"))
    }
}

pub fn new_keypair() -> Result<(String, String), PtsError> {
    let (PublicKey(public_key), PrivateKey(private_key)) = box_::gen_keypair();
    let public_key_rep = try!(public_key.to_rfc1751()
        .map_err(|e| PE::Fatal("Failed to encode generated public key")))
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
