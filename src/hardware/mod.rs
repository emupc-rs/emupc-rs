use crate::cpu8086::*;
use crate::ibmpc5150machine::*;

use crate::cpu286::*;
use crate::ibmpcatmachine::*;

pub mod ibmpc5150machine;
pub mod ibmpcatmachine;
pub mod pit;

#[derive(Clone, Debug, Default)]
pub struct IbmPc5150Machine {
    pub cpu: Cpu8086,
    pub hardware: IbmPc5150Hardware,
}

impl IbmPc5150Machine {
    pub fn new() -> IbmPc5150Machine {
        IbmPc5150Machine {
            cpu: Cpu8086::new(),
            hardware: IbmPc5150Hardware::new(),
        }
    }
    pub fn tick(&mut self, cycles: usize) {
        self.hardware.tick(cycles);
    }
}

#[derive(Clone, Debug, Default)]
pub struct IbmPcAtMachine {
    pub cpu: Cpu286,
    pub hardware: IbmPcAtHardware,
}

impl IbmPcAtMachine {
    pub fn new() -> IbmPcAtMachine {
        IbmPcAtMachine {
            cpu: Cpu286::new(),
            hardware: IbmPcAtHardware::new(),
        }
    }
}
