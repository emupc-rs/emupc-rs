extern crate bitflags;

mod registers;

use registers::*;
use std::fs;

#[allow(dead_code)]

pub trait CpuContext {
    fn mem_read_byte(&mut self, addr: u32) -> u8;
    fn mem_write_byte(&mut self, addr: u32, value: u8);
    fn io_read_byte(&mut self, addr: u16) -> u8;
    fn io_write_byte(&mut self, addr: u16, value: u8);
}

#[derive(Clone, Copy, Debug)]
pub struct Cpu {
    pub regs : Registers,
    pub opcode : u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: Registers::new(),
            opcode: 0,
        }
    }
    pub fn mem_read_byte<T: CpuContext>(&mut self, ctx: &mut T, seg: u16, addr: u16) -> u8 {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xfffff;
        ctx.mem_read_byte(masked_addr)
    }
    pub fn mem_write_byte<T: CpuContext>(&mut self, ctx: &mut T, seg: u16, addr: u16, value: u8) {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xfffff;
        ctx.mem_write_byte(masked_addr, value)
    }

    pub fn mem_read_word<T: CpuContext>(&mut self, ctx: &mut T, seg: u16, addr: u16) -> u16 {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xfffff;
        let byte1 = ctx.mem_read_byte(masked_addr) as u16;
        let byte2 = (ctx.mem_read_byte((masked_addr + 1) & 0xfffff) as u16) << 8;
        byte1 | byte2
    }

    pub fn tick<T: CpuContext>(&mut self, ctx: &mut T) {
        self.opcode = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip);
        println!("Opcode {:#02x}", self.opcode);
        match self.opcode {
            0xea => {
                println!("jmp far");
                let offset = self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip + 1);
                let segment = self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip + 3);
                self.regs.writeseg16(SegReg::CS, segment);
                self.regs.ip = offset;
            },
            _ => panic!("Unhandled opcode!"),
        }
    }
}



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
