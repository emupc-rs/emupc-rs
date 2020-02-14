use crate::cpu8086::*;
use crate::ibmpc5150machine::*;

pub mod ibmpc5150machine;

#[derive(Clone, Debug)]
pub struct IbmPc5150Machine {
    pub cpu: Cpu8086,
    pub hardware: IbmPc5150Hardware,
}

impl IbmPc5150Machine {
    pub fn new() -> IbmPc5150Machine {
        IbmPc5150Machine {
            cpu: Cpu8086::new(),
            hardware: IbmPc5150Hardware::new()
        }
    }
}