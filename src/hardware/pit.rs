#[derive(Clone, Copy, PartialEq)]
enum AccessMode {
    HighThenLow = 0,
    AlwaysLow = 1,
    AlwaysHigh = 2,
    LowThenHigh = 3
}

#[derive(Clone, Copy)]
pub struct PitCounter {
    pub timer_mode: u8,
    pub access_mode: AccessMode,
    pub gate: bool,
    pub out: bool,
}

#[derive(Clone, Copy)]
pub struct PIT {
    pub counters: [PitTimer; 3],
}