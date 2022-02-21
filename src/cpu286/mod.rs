use crate::cpu286::registers::*;

pub mod registers;

pub trait Cpu286Context {
    fn mem_read_byte(&mut self, addr: u32) -> u8;
    fn mem_write_byte(&mut self, addr: u32, value: u8);
    fn io_read_byte(&mut self, addr: u16) -> u8;
    fn io_write_byte(&mut self, addr: u16, value: u8);
}

#[derive(Clone, Debug, Default)]
pub struct Cpu286 {
    pub regs: Registers,
    pub opcode: u8,
    pub floppy: Vec<u8>,
}

impl Cpu286 {
    pub fn new() -> Cpu286 {
        Cpu286 {
            regs: Registers::new(),
            opcode: 0,
            floppy: vec![],
        }
    }
    pub fn mem_read_byte<T: Cpu286Context>(&mut self, ctx: &mut T, addr: u32) -> u8 {
        let masked_addr = addr & 0xff_ffff;
        ctx.mem_read_byte(masked_addr)
    }
    pub fn mem_write_byte<T: Cpu286Context>(&mut self, ctx: &mut T, addr: u32, value: u8) {
        let masked_addr = addr & 0xff_ffff;
        ctx.mem_write_byte(masked_addr, value)
    }

    pub fn mem_read_word<T: Cpu286Context>(&mut self, ctx: &mut T, addr: u32) -> u16 {
        let masked_addr = addr & 0xff_ffff;
        let lo = ctx.mem_read_byte(masked_addr);
        let hi = ctx.mem_read_byte(masked_addr.wrapping_add(1) & 0xff_ffff);
        u16::from_le_bytes([lo, hi])
    }

    pub fn tick<T: Cpu286Context>(&mut self, ctx: &mut T) -> usize {
        self.opcode = self.mem_read_byte(
            ctx,
            self.regs.readseg16(SegReg::CS).base + self.regs.ip as u32,
        );
        println!(
            "Opcode {:#02x} CS base {:#06x} IP {:#04x}",
            self.opcode,
            self.regs.readseg16(SegReg::CS).base,
            self.regs.ip
        );
        match self.opcode {
            0x70 => {
                println!("jo");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if self.regs.flags.contains(Flags::OVERFLOW) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x71 => {
                println!("jno");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if !self.regs.flags.contains(Flags::OVERFLOW) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x72 => {
                println!("jc");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if self.regs.flags.contains(Flags::CARRY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x73 => {
                println!("jnc");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if !self.regs.flags.contains(Flags::CARRY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x74 => {
                println!("jz");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if self.regs.flags.contains(Flags::ZERO) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x75 => {
                println!("jnz");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if !self.regs.flags.contains(Flags::ZERO) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x78 => {
                println!("js");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if self.regs.flags.contains(Flags::SIGN) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x79 => {
                println!("jns");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if !self.regs.flags.contains(Flags::SIGN) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x7a => {
                println!("jp");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if self.regs.flags.contains(Flags::PARITY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x7b => {
                println!("jnp");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(1);
                if !self.regs.flags.contains(Flags::PARITY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x9e => {
                println!("sahf");
                self.regs.flags = Flags::from_bits(
                    (self.regs.flags.bits() & 0xff02) | (self.regs.read8(Reg8::AH) as u16),
                )
                .unwrap();
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x9f => {
                println!("lahf");
                self.regs.write8(
                    Reg8::AH,
                    ((self.regs.flags.bits() & 0xd5) | (0x0002_u16)) as u8,
                ); //Flags::DEFAULT
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xb0 => {
                println!("mov al, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::AL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb1 => {
                println!("mov cl, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::CL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb2 => {
                println!("mov dl, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::DL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb3 => {
                println!("mov bl, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::BL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb4 => {
                println!("mov ah, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::AH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb5 => {
                println!("mov ch, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::CH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb6 => {
                println!("mov dh, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::DH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb7 => {
                println!("mov bh, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.write8(Reg8::BH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xe9 => {
                println!("jmp near");
                let offset = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                self.regs.ip = self.regs.ip.wrapping_add(offset);
            }
            0xea => {
                println!("jmp far");
                let offset = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(1) as u32,
                );
                let segment = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base + self.regs.ip.wrapping_add(3) as u32,
                );
                self.regs.writeseg16(SegReg::CS, segment);
                self.regs.ip = offset;
            }
            0xeb => {
                println!("jmp rel8");
                let offset = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS).base +
                    self.regs.ip.wrapping_add(1) as u32
                );
                self.regs.ip = self.regs.ip.wrapping_add((offset as i8 as u16) + 2u16);
            }
            0xfa => {
                println!("cli");
                self.regs.flags.set(Flags::INTERRUPT, false);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xfb => {
                println!("sti");
                self.regs.flags.set(Flags::INTERRUPT, true);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            _ => panic!("Unhandled opcode!"),
        }
        2
    }
}
