#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessMode {
    HighThenLow = 0,
    AlwaysLow = 1,
    AlwaysHigh = 2,
    LowThenHigh = 3
}

#[derive(Debug, Clone, Copy)]
pub struct PitCounter {
    pub timer_mode: u8,
    pub access_mode: AccessMode,
    pub gate: bool,
    pub out: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct PIT {
    pub counters: [PitCounter; 3],
}

impl PIT {
    pub fn new() -> Self {
        Self {
            counters: [PitCounter {
                timer_mode: 0,
                access_mode: AccessMode::LowThenHigh,
                gate: false,
                out: false,
            }; 3]
        }
    }
    pub fn tick(&mut self, _cycles: usize) {
        println!("PIT TICKED");
    }
}

impl Default for PIT {
    fn default() -> PIT {
        PIT::new()
    }
}