use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type I32 = i32;
pub type U32 = u32;

#[derive(Clone, Copy, Debug)]
pub struct F32(pub f32);

#[cfg(test)]
impl Eq for F32 {}

#[cfg(test)]
impl PartialEq for F32 {
    /// Precision issues make it difficult to compare floats.
    /// Since we are in development version, we'll use what works and not worry too much about
    /// the details, but good references can be found [here (1)], [here (2)] and [here (3)].
    ///
    /// [here (1)]: https://randomascii.wordpress.com/2012/02/25/comparing-floating-point-numbers-2012-edition/
    /// [here (2)]: http://blog.reverberate.org/2014/09/what-every-computer-programmer-should.html
    /// [here (3)]: https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl From<f32> for F32 {
    fn from(this: f32) -> Self {
        F32(this)
    }
}

impl FromStr for F32 {
    type Err = std::num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
    }
}

impl Default for F32 {
    fn default() -> Self {
        Self(f32::default())
    }
}

trait Inc {
    fn increment(&self) -> usize;
}

const MONOTONIC: Ordering = Ordering::Relaxed;

impl Inc for AtomicUsize {
    fn increment(&self) -> usize {
        let now = self.load(MONOTONIC);

        if now == std::usize::MAX {
            panic!("Counter was exhausted.")
        }

        let next = now + 1;
        self.store(next, MONOTONIC);
        next
    }
}