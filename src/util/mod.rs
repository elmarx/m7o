use rand::Rng;
use rand::distr::Alphanumeric;

pub mod cm_ext;
pub mod hash;

pub fn generate_password() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect()
}
