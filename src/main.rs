extern crate bitflags;

use crate::cpu8086::*;
use crate::hardware::*;

pub mod cpu8086;
pub mod hardware;

#[allow(dead_code)]

fn main() {
    let mut machine = IbmPc5150Machine::new();

    //machine.cpu.tick(&mut machine.hardware);
    loop {
        machine.cpu.tick(&mut machine.hardware);
    }
}
