use procgen::pine_plantation::Map1;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn main() {
    let mut rng1 = rand::rng();
    let rng_seed = rng1.random::<u64>();
    //    let rng_seed = 11251360553627691406;
    println!("Rng seed: {}", rng_seed);
    let mut rng = StdRng::seed_from_u64(rng_seed);
    let map = Map1::generate(&mut rng);
    map.print();
}
