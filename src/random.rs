use core::ptr;
use core::result::Result;
use bit_field::BitField;

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
    pub fn new() -> MTRng32 {
        let mut rng = MTRng32 {
            state: [0; N],
            // index = N + 1 means state is not initialized.
            index: N + 1,
        };

        // get seed from hardware random register
        let mut hwr = HwRng::init().unwrap();
        loop {
            match hwr.poll_and_get() {
                Ok(v) => {
                    rng.reset(v);
                    break;
                }
                Err(_) => continue,
            }
        }

        rng
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

impl Rng for MTRng32 {
    fn rand(&mut self) -> u32 {
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
}

impl Default for MTRng32 {
    fn default() -> Self {
        Self::new()
    }
}

// Access Hardware Random Register to Seed the
// pseudo random number generator
const RNG_BASE_ADDR: u32 = 0x5006_0800;
const RNG_CR: u32 = RNG_BASE_ADDR;
const RNG_STATUS: u32 = RNG_BASE_ADDR + 0x4;
const RNG_DATA: u32 = RNG_BASE_ADDR + 0x8;

const RCC_BASE_ADDR: u32 = 0x4002_3800;
const RCC_AHB2ENR: u32 = RCC_BASE_ADDR + 0x34;

struct HwRng(u32, u32);

#[derive(Debug)]
enum ErrorType {
    CECS,
    SECS,
    CEIS,
    SEIS,
    AlreadyEnabled,
    NotReady,
}

impl HwRng {
    pub fn init() -> Result<Self, ErrorType> {
        let reg_content = unsafe { ptr::read_volatile(RNG_CR as *mut u32) };
        if reg_content.get_bit(2) {
            return Err(ErrorType::AlreadyEnabled);
        }

        Self::enable_cr();
        Ok(HwRng(0x0, 0x0))
    }

    pub fn poll_and_get(&mut self) -> Result<u32, ErrorType> {
        let status = unsafe { ptr::read_volatile(RNG_STATUS as *mut u32) };

        if status.get_bit(5) {
            Self::disable_cr();
            Self::enable_cr();
            return Err(ErrorType::CEIS);
        }
        if status.get_bit(6) {
            Self::disable_cr();
            Self::enable_cr();
            return Err(ErrorType::SEIS);
        }

        if status.get_bit(1) {
            return Err(ErrorType::CECS);
        }
        if status.get_bit(2) {
            Self::disable_cr();
            Self::enable_cr(); // recommended by manual
            return Err(ErrorType::SECS);
        }
        if status.get_bit(0) {
            let data = unsafe { ptr::read_volatile(RNG_DATA as *mut u32) };
            if data != self.0 {
                self.0 = data;
                self.1 = 0;
                return Ok(data);
            }
        }

        self.1 += 1;
        if self.1 > 80 {
            Self::disable_cr();
            Self::enable_cr();
            self.1 = 0;
        }

        // data was not ready, try again!
        Err(ErrorType::NotReady)
    }

    fn enable_cr() {
        let mut bits_rcc_en: u32 = 0;
        bits_rcc_en.set_bit(6, true);

        let mut bits_rng_cr: u32 = 0;
        bits_rng_cr.set_bit(2, true);

        unsafe {
            // clock enable
            ptr::write_volatile(RCC_AHB2ENR as *mut u32, bits_rcc_en);

            // device enable
            ptr::write_volatile(RNG_CR as *mut u32, bits_rng_cr);
        }
    }


    fn disable_cr() {
        let mut bits = unsafe { ptr::read_volatile(RNG_CR as *mut u32) };
        bits.set_bit(2, false);
        bits.set_bit(3, false);

        unsafe {
            ptr::write_volatile(RNG_CR as *mut u32, bits);
            assert_eq!(ptr::read_volatile(RNG_CR as *mut u32), bits);
        };
    }
}
