#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessMode {
    HighThenLow = 0,
    AlwaysLow = 1,
    AlwaysHigh = 2,
    LowThenHigh = 3,
}

pub enum PitCtrState {}

#[derive(Debug, Clone, Copy)]
pub struct PitCounter {
    pub timer_mode: u8,
    pub access_mode: AccessMode,
    pub count: u16,
    pub gate: bool,
    pub out: bool,
}

pub enum PitType {
    PIT8253,
    PIT8254,
}

#[derive(Debug, Clone, Copy)]
pub struct PIT {
    pub counters: [PitCounter; 3],
    pub ctrl: u8,
}

impl PIT {
    pub fn new() -> Self {
        Self {
            counters: [PitCounter {
                timer_mode: 0,
                access_mode: AccessMode::LowThenHigh,
                count: 0xffffu16,
                gate: false,
                out: false,
            }; 3],
            ctrl: 0,
        }
    }
    pub fn tick(&mut self, _cycles: usize) {
        println!("PIT TICKED");
    }

    pub fn rb(&mut self, _addr: u16) -> u8 {
        0
    }

    pub fn wb(&mut self, addr: u16, data: u8) {
        match addr & 3 {
            3 => {
                if (data >> 6) == 3 {
                    todo!();
                } else {
                    self.ctrl = data;
                }
            }
            _ => todo!(),
        }
    }
}

impl Default for PIT {
    fn default() -> PIT {
        PIT::new()
    }
}
