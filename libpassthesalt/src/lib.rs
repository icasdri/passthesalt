extern crate sodiumoxide as sodium;

use sodium::crypto::box_;

pub enum PtsError {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
