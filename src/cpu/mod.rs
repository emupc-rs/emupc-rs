use crate::registers::*;

pub mod registers;

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
        let byte2 = (ctx.mem_read_byte(masked_addr.wrapping_add(1) & 0xfffff) as u16) << 8;
        byte1 | byte2
    }

    pub fn tick<T: CpuContext>(&mut self, ctx: &mut T) {
        self.opcode = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip);
        println!("Opcode {:#02x}", self.opcode);
        match self.opcode {
            0xea => {
                println!("jmp far");
                let offset = self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip.wrapping_add(1));
                let segment = self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip.wrapping_add(3));
                self.regs.writeseg16(SegReg::CS, segment);
                self.regs.ip = offset;
            },
            _ => panic!("Unhandled opcode!"),
        }
    }
}