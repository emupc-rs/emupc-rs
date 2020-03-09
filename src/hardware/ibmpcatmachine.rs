use crate::cpu286::*;
use std::fs;

#[derive(Clone, Debug, Default)]
pub struct IbmPcAtHardware {
    pub ram: Vec<u8>,
    pub bios_rom: Vec<u8>,
}

impl IbmPcAtHardware {
    pub fn new() -> IbmPcAtHardware {
        IbmPcAtHardware {
            ram: vec![0; 0xa0000],
            bios_rom: {
                let low_rom: Vec<u8> =
                    fs::read("roms/machines/ibmatami/BIOS_5170_30APR89_U27_AMI_27256.BIN").unwrap();
                let high_rom: Vec<u8> =
                    fs::read("roms/machines/ibmatami/BIOS_5170_30APR89_U47_AMI_27256.BIN").unwrap();

                let mut bios: Vec<u8> = vec![0; 0x10000];

                for i in 0..0x8000 {
                    bios[(i << 1)] = low_rom[i];
                    bios[(i << 1) + 1] = high_rom[i];
                }
                bios
            },
        }
    }
}

impl<'a> Cpu286Context for IbmPcAtHardware {
    fn mem_read_byte(&mut self, addr: u32) -> u8 {
        let actual_addr = addr & 0xff_ffff;
        match actual_addr {
            0..=0x0a_0000 => self.ram[(actual_addr & 0xffff) as usize],
            0x0f_0000..=0x0f_ffff => self.bios_rom[(actual_addr & 0xffff) as usize],
            0xff_0000..=0xff_ffff => self.bios_rom[(actual_addr & 0xffff) as usize],
            _ => 0xff,
        }
    }
    fn mem_write_byte(&mut self, addr: u32, value: u8) {
        let actual_addr = addr & 0xff_ffff;
        if let 0..=0x0a_0000 = actual_addr { self.ram[(actual_addr & 0xffff) as usize] = value }
    }

    fn io_read_byte(&mut self, _addr: u16) -> u8 {
        0xff
    }

    fn io_write_byte(&mut self, _addr: u16, _value: u8) {
    }
}
