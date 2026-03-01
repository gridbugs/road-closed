use procgen::city::{Map, TentacleSpec};
use rand::{Rng, SeedableRng, rngs::StdRng};

fn main() {
    let mut rng1 = rand::rng();
    let rng_seed = rng1.random::<u64>();
    //    let rng_seed = 11251360553627691406;
    println!("Rng seed: {}", rng_seed);
    let mut rng = StdRng::seed_from_u64(rng_seed);
    let tentacle_spec = TentacleSpec {
        num_tentacles: 3,
        segment_length: 2.0,
        distance_from_centre: 40.0,
        spread: 0.3,
    };
    let map = Map::generate(&tentacle_spec, &mut rng);
    map.print();
}
