// Multiply-with-carry Algorithm
// https://en.wikipedia.org/wiki/Multiply-with-carry#Complementary-multiply-with-carry_generators

use stm32f7::system_clock;

// CMWC working parts
const CMWC_CYCLE: usize = 512; // as Marsaglia recommends
const CMWC_C_MAX: u32 = 809430660; // as Marsaglia recommends

pub struct CMWC_State {
    q: [u32; CMWC_CYCLE],
    c: u32, // must be limited with CMWC_C_MAX
    i: u32,
}

impl CMWC_State {
    // Init the state with seed
    pub fn new(seed: u32) -> Self {
        let mut state = CMWC_State {
            q: [0; CMWC_CYCLE],
            c: 0,
            i: 0,
        };

        for i in 0..CMWC_CYCLE {
            state.q[i] = rand_u32();
        }

        state.c = rand_u32();
        while state.c >= CMWC_C_MAX {
            state.c = rand_u32();
        }
        state.i = (CMWC_CYCLE as u32) - 1;

        state
    }

    // CMWC engine
    pub fn rand(&mut self) -> u32 {
        let a: u32 = 18782; // as Marsaglia recommends
        let m: u32 = 0xfffffffe; // as Marsaglia recommends
        let t: u32;
        let mut x: u32;

        self.i = (self.i + 1) & (CMWC_CYCLE as u32 - 1);
        t = a.wrapping_mul(self.q[self.i as usize]).wrapping_add(self.c);
        self.c = t;
        x = t.wrapping_add(self.c);

        if x < self.c {
            x += 1;
            self.c += 1;
        }

        self.q[self.i as usize] = m.wrapping_sub(x);
        self.q[self.i as usize]
    }

    pub fn get_random_pos(&mut self, width: u16, height: u16) -> (u16, u16) {
        (self.rand() as u16 % width, self.rand() as u16 % height)
    }
}

// Make 32 bit random number (some systems use 16 bit RAND_MAX [Visual C 2012 uses 15 bits!])
fn rand_u32() -> u32 {
    let result = system_clock::ticks() as u32;
    result << 16 | (system_clock::ticks() as u32)
}