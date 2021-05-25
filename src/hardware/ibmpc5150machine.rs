use crate::cpu8086::*;
use crate::hardware::pit::*;
use std::fs;

#[derive(Clone, Debug, Default)]
pub struct IbmPc5150Hardware {
    pub ram: Vec<u8>,
    pub bios_rom: Vec<u8>,
    pub pit: PIT,
}

impl IbmPc5150Hardware {
    pub fn new() -> IbmPc5150Hardware {
        IbmPc5150Hardware {
            ram: vec![0; 0x10000],
            bios_rom: fs::read("roms/machines/ibmpc/BIOS_5150_24APR81_U33.BIN").unwrap(),
            pit: PIT::new(),
        }
    }
    pub fn tick(&mut self, cycles: usize) {
        self.pit.tick(cycles);
    }
}

impl<'a> Cpu8086Context for IbmPc5150Hardware {
    fn mem_read_byte(&mut self, addr: u32) -> u8 {
        let actual_addr = addr & 0xf_ffff;
        match actual_addr {
            0..=0x1_0000 => self.ram[(actual_addr & 0xffff) as usize],
            0xf_e000..=0xf_ffff => self.bios_rom[(actual_addr & 0x1fff) as usize],
            _ => 0xff,
        }
    }
    fn mem_write_byte(&mut self, addr: u32, value: u8) {
        let actual_addr = addr & 0xf_ffff;
        if let 0..=0x0a_0000 = actual_addr {
            self.ram[(actual_addr & 0xffff) as usize] = value
        }
    }

    fn io_read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0040..=0x0043 => self.pit.rb(addr),
            _ => {
                println!("Unimplemented IO read");
                0xff
            }
        }
    }

    fn io_write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0040..=0x0043 => self.pit.wb(addr, value),
            _ => println!("Unimplemented IO write"),
        }
    }
}
