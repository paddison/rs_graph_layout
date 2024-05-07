use std::time::SystemTime;

const LCG_MULTIPLIER: usize = 0x5deece66d;
const LCG_INCREMENT: usize = 0x5deece66d;

struct LCG {
    state: u128,
    a: usize,
    c: usize,
}

impl LCG {
    pub fn new() -> Self {
        // generater state from clock
        let state = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self {
            state,
            a: LCG_MULTIPLIER,
            c: LCG_INCREMENT,
        }
    }

    pub fn new_seed(seed: u128) -> Self {
        Self {
            state: seed,
            a: LCG_MULTIPLIER,
            c: LCG_INCREMENT,
        }
    }

    fn next(&mut self) -> u128 {
        self.state = self
            .state
            .wrapping_mul(self.a as u128)
            .wrapping_sub(self.c as u128);
        self.state
    }

    fn generate_range(&mut self, range: usize) -> usize {
        ((self.next() >> 64) % range as u128) as usize
    }
}


#[test]
fn test_lcg_generate_range() {
    let mut lcg = LCG::new();
    let next = lcg.generate_range(10);

    assert!(next < 10);}
