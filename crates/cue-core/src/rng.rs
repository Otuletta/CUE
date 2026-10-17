/// A small, seedable, platform-independent pseudo-random generator.
///
/// SplitMix64 (Steele, Lea & Flood, 2014). It is used only where the game
/// needs reproducible randomness, such as rack shuffles and future challenge
/// layouts. Its output depends on nothing but the seed, so a seed recorded in
/// a replay produces the same rack on every platform, including WebAssembly.
///
/// It is **not** cryptographically secure.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a generator from a seed.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform integer in `0..bound`, without modulo bias.
    ///
    /// # Panics
    ///
    /// Panics if `bound` is zero.
    pub fn below(&mut self, bound: u64) -> u64 {
        assert!(bound > 0, "bound must be positive");
        // Reject the final partial bucket so every residue is equally likely.
        let zone = u64::MAX - (u64::MAX % bound);
        loop {
            let x = self.next_u64();
            if x < zone {
                return x % bound;
            }
        }
    }

    /// Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_reference_sequence() {
        // Reference values for seed 0 from the original SplitMix64 algorithm.
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(rng.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = SplitMix64::new(42);
        for bound in 1..50 {
            for _ in 0..100 {
                assert!(rng.below(bound) < bound);
            }
        }
    }

    #[test]
    fn shuffle_is_a_permutation_and_reproducible() {
        let mut a: Vec<u32> = (0..20).collect();
        let mut b = a.clone();
        SplitMix64::new(7).shuffle(&mut a);
        SplitMix64::new(7).shuffle(&mut b);
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>());
    }
}
