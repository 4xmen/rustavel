use rand;
use rand::RngExt;

pub fn random(min: i32, max: i32) -> i32 {
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}