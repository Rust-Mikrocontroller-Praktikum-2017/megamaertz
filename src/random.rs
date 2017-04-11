pub trait Rng {
    fn rand(&mut self) -> u32;
}

// Mersenne Twister 32-bits implementation.
//
// See [Mersenne Twister Homepage]
//(http://www.math.sci.hiroshima-u.ac.jp/~m-mat/MT/MT2002/emt19937ar.html)
// for more informations.
//
// See: https://github.com/KokaKiwi/rust-mersenne-twister/blob/master/src/mt32.rs
const N: usize = 624;
const M: usize = 397;
const DIFF: isize = M as isize - N as isize;

const MAGIC: [u32; 2] = [0, 0x9908b0df];

const MAGIC_VALUE1: u32 = 1812433253;
const MAGIC_VALUE2: u32 = 0x9d2c5680;
const MAGIC_VALUE3: u32 = 0xefc60000;

const UPPER_MASK: u32 = 1 << 31;
const LOWER_MASK: u32 = !UPPER_MASK;


pub struct MTRng32 {
    state: [u32; N],
    index: usize,
}

impl MTRng32 {
    pub fn new(seed: u32) -> MTRng32 {
        let mut rng = MTRng32 {
            state: [0; N],
            // index = N + 1 means state is not initialized.
            index: N + 1,
        };

        rng.reset(seed);
        rng
    }

    pub fn rand(&mut self) -> u32 {
        if self.index >= N {
            self.generate_words();
        }

        let mut y = self.state[self.index];
        self.index += 1;

        y ^= y >> 11;
        y ^= (y << 7) & MAGIC_VALUE2;
        y ^= (y << 15) & MAGIC_VALUE3;
        y ^= y >> 18;

        y
    }

    fn reset(&mut self, seed: u32) {
        self.state[0] = seed;
        for index in 1..N {
            let prec = self.state[index - 1];

            self.state[index] = MAGIC_VALUE1.wrapping_mul(prec ^ (prec >> 30)) + index as u32;
        }
        self.index = N;
    }

    fn generate_words(&mut self) {
        for index in 0..(N - M) {
            let y = (self.state[index] & UPPER_MASK) | (self.state[index + 1] & LOWER_MASK);
            let magic_idx = (y & 0x1) as usize;
            self.state[index] = self.state[index + M] ^ (y >> 1) ^ MAGIC[magic_idx];
        }

        for index in (N - M)..(N - 1) {
            let y = (self.state[index] & UPPER_MASK) | (self.state[index + 1] & LOWER_MASK);
            let magic_idx = (y & 0x1) as usize;
            let nindex = index as isize + DIFF;
            self.state[index] = self.state[nindex as usize] ^ (y >> 1) ^ MAGIC[magic_idx];
        }

        {
            let y = (self.state[N - 1] & UPPER_MASK) | (self.state[0] & LOWER_MASK);
            let magic_idx = (y & 0x1) as usize;
            self.state[N - 1] = self.state[M - 1] ^ (y >> 1) ^ MAGIC[magic_idx];
        }

        self.index = 0;
    }
}
