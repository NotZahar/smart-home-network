use rand::RngExt;
use std::marker::PhantomData;

use crate::utils::trait_alias::RandomNumber;

pub trait RandomGenerator<T: RandomNumber> {
    fn new() -> Self;
    fn generate(&mut self, min: T, max: T) -> T;
}

#[derive(Debug)]
pub struct SimpleRandomGenerator<T: RandomNumber> {
    generator: rand::rngs::SmallRng,
    _phantom_for_t: PhantomData<T>,
}

impl<T: RandomNumber> RandomGenerator<T> for SimpleRandomGenerator<T> {
    fn new() -> Self {
        SimpleRandomGenerator {
            generator: rand::make_rng(),
            _phantom_for_t: PhantomData,
        }
    }

    fn generate(&mut self, min: T, max: T) -> T {
        assert!(min < max);
        self.generator.random_range(min..=max)
    }
}
