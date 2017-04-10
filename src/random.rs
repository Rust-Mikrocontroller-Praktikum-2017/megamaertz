use core::cmp;
use core::ptr;
use core::result::Result;
use bit_field::BitField;

use stm32f7::system_clock;
use constants;


// Multiply-with-carry Algorithm
// https://en.wikipedia.org/wiki/Multiply-with-carry#Complementary-multiply-with-carry_generators
pub struct CmwcState {
    q: [u32; constants::CMWC_CYCLE],
    c: u32, // must be limited with constants::CMWC_C_MAX
    i: u32,
}

impl CmwcState {
    // Init the state with seed
    pub fn new() -> Self {
        let mut state = CmwcState {
            q: [0; constants::CMWC_CYCLE],
            c: 0,
            i: 0,
        };

        for i in 0..constants::CMWC_CYCLE {
            state.q[i] = rand_u32();
        }

        let mut hwr = HwRng::init().unwrap();
        loop {
            match hwr.poll_and_get() {
                Ok(v) => {
                    state.c = v;
                    break;
                }
                Err(_) => continue,
            }
        }

        state.i = (constants::CMWC_CYCLE as u32) - 1;

        state
    }

    // CMWC engine
    pub fn rand(&mut self) -> u32 {
        let a: u32 = 18782; // as Marsaglia recommends
        let m: u32 = 0xfffffffe; // as Marsaglia recommends
        let t: u32;
        let mut x: u32;

        self.i = (self.i + 1) & (constants::CMWC_CYCLE as u32 - 1);
        t = a.wrapping_mul(self.q[self.i as usize])
            .wrapping_add(self.c);
        self.c = t;
        x = t.wrapping_add(self.c);

        if x < self.c {
            x += 1;
            self.c += 1;
        }

        self.q[self.i as usize] = m.wrapping_sub(x);
        self.q[self.i as usize]
    }

    pub fn get_random_pos(&mut self, target_width: u16, target_height: u16) -> (u16, u16) {
        (core::cmp::min(self.rand() as u16 % constants::DISPLAY_SIZE.0 - 1,
                        constants::DISPLAY_SIZE.0 - target_width - 1),
         core::cmp::min(self.rand() as u16 % constants::DISPLAY_SIZE.1 - 1,
                        constants::DISPLAY_SIZE.1 - target_height - 1))
    }
}

impl Default for CmwcState {
    fn default() -> Self {
        Self::new()
    }
}

// Make 32 bit random number (some systems use 16 bit RAND_MAX [Visual C 2012 uses 15 bits!])
fn rand_u32() -> u32 {
    let result = system_clock::ticks() as u32;
    result << 16 | (system_clock::ticks() as u32)
}

// Access Hardware Random Register to Seed the
// pseudo random number generator
const RNG_BASE_ADDR: u32 = 0x5006_0800;
const RNG_CR: u32 = RNG_BASE_ADDR + 0x0;
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

