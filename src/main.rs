extern crate bitflags;

use crate::cpu::*;
use std::fs;

pub mod cpu;

#[allow(dead_code)]

#[derive(Clone, Debug)]
pub struct IbmPc5150Hardware {
    pub ram : Vec<u8>,
    pub bios_rom : Vec<u8>,
}

impl IbmPc5150Hardware {
    pub fn new() -> IbmPc5150Hardware {
        IbmPc5150Hardware {
            ram: vec![0; 0x10000],
            bios_rom: fs::read("roms/machines/ibmpc/BIOS_5150_24APR81_U33.BIN").unwrap(),
        }
    }
}

impl<'a> CpuContext for IbmPc5150Hardware {
    fn mem_read_byte(&mut self, addr: u32) -> u8 {
        let actual_addr = addr & 0xfffff;
        match actual_addr {
            0 ..= 0x10000 => self.ram[(addr & 0xffff) as usize],
            0xfe000 ..= 0xfffff => self.bios_rom[(addr & 0x1fff) as usize],
            _ => 0xff,
        }
    }
    fn mem_write_byte(&mut self, addr: u32, value: u8) {
        let actual_addr = addr & 0xfffff;
        match actual_addr {
            0 ..= 0x10000 => self.ram[(addr & 0xffff) as usize] = value,
            _ => return,
        };
    }

    fn io_read_byte(&mut self, addr: u16) -> u8 {
        0xff
    }

    fn io_write_byte(&mut self, addr: u16, value: u8) {
        return
    }
}

#[derive(Clone, Debug)]
pub struct IbmPc5150Machine {
    pub cpu: Cpu,
    pub hardware: IbmPc5150Hardware,
}

impl IbmPc5150Machine {
    pub fn new() -> IbmPc5150Machine {
        IbmPc5150Machine {
            cpu: Cpu::new(),
            hardware: IbmPc5150Hardware::new()
        }
    }
}

fn main() {
    let mut machine = IbmPc5150Machine::new();

    machine.cpu.tick(&mut machine.hardware);
    machine.cpu.tick(&mut machine.hardware);
}
