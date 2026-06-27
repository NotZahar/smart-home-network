use num_traits::{ConstZero, Num};
use rand::distr::uniform::SampleUniform;

pub trait Number: Num + ConstZero + Copy + PartialOrd {}
impl<T: Num + ConstZero + Copy + PartialOrd> Number for T {}

pub trait RandomNumber: Number + SampleUniform {}
impl<T: Number + SampleUniform> RandomNumber for T {}
