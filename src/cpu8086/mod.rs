//use crate::scheduler::Jiffies;
use operand::*;
use registers::*;

pub mod operand;
pub mod registers;

pub trait Cpu8086Context {
    fn mem_read_byte(&mut self, addr: u32) -> u8;
    fn mem_write_byte(&mut self, addr: u32, value: u8);
    fn io_read_byte(&mut self, addr: u16) -> u8;
    fn io_write_byte(&mut self, addr: u16, value: u8);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RepType {
    REPE,
    REPNE
}

#[derive(Clone, Debug, Default)]
pub struct Cpu8086 {
    pub regs: Registers,
    pub opcode: u8,
    pub seg_override: Option<SegReg>,
    pub rep_state: Option<RepType>,
    pub floppy: Vec<u8>
}

impl Cpu8086 {
    pub fn new() -> Cpu8086 {
        Cpu8086 {
            regs: Registers::new(),
            opcode: 0,
            seg_override: None,
            rep_state: None,
            floppy: vec![],
        }
    }
    pub fn interrupt_hook<T: Cpu8086Context>(&mut self, ctx: &mut T, intr: u8) {
        match intr {
            0x10 => {
                match self.regs.read8(Reg8::AH) {
                    0x0e => {
                        println!("Teletype output");
                        println!("{}", self.regs.read8(Reg8::AL) as char);
                    }
                    _ => panic!("Unimplemented int 10"),
                }
            }
            0x13 => {
                match self.regs.read8(Reg8::AH) {
                    0x00 => {
                        println!("reset disk system");
                        self.regs.write8(Reg8::AH, 0);
                        self.regs.flags.set(Flags::CARRY, false);
                    },
                    0x02 => {
                        println!("read sectors");
                        let count: u16 = self.regs.read8(Reg8::AL) as u16;
                        let sector: u16 = self.regs.read8(Reg8::CL) as u16;
                        let buf_seg = self.regs.readseg16(SegReg::ES);
                        let buf_off = self.regs.read16(Reg16::BX);
                        for i in 0..=(count-1) {
                            for j in 0..=511 {
                                self.mem_write_byte(ctx, buf_seg, buf_off.wrapping_add((i << 9)+j), self.floppy[(((sector+i)<<9)+j) as usize]);
                            }
                        }
                        self.regs.flags.set(Flags::CARRY, false);
                        self.regs.write8(Reg8::AH, 0);
                        self.regs.write8(Reg8::AL, count as u8);
                    },
                    _ => panic!("Unimplmented int 13"),
                }
            }
            _ => panic!("Unimplemented interrupt"),
        }
    }
    pub fn mem_read_byte<T: Cpu8086Context>(&mut self, ctx: &mut T, seg: u16, addr: u16) -> u8 {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xf_ffff;
        ctx.mem_read_byte(masked_addr)
    }
    pub fn mem_write_byte<T: Cpu8086Context>(
        &mut self,
        ctx: &mut T,
        seg: u16,
        addr: u16,
        value: u8,
    ) {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xf_ffff;
        ctx.mem_write_byte(masked_addr, value)
    }

    pub fn io_read_byte<T: Cpu8086Context>(&mut self, ctx: &mut T, addr: u16) -> u8 {
        ctx.io_read_byte(addr)
    }

    pub fn io_write_byte<T: Cpu8086Context>(&mut self, ctx: &mut T, addr: u16, value: u8) {
        ctx.io_write_byte(addr, value)
    }

    pub fn mem_read_word<T: Cpu8086Context>(&mut self, ctx: &mut T, seg: u16, addr: u16) -> u16 {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xf_ffff;
        let lo = ctx.mem_read_byte(masked_addr);
        let hi = ctx.mem_read_byte(masked_addr.wrapping_add(1) & 0xf_ffff);
        u16::from_le_bytes([lo, hi])
    }

    pub fn mem_write_word<T: Cpu8086Context>(
        &mut self,
        ctx: &mut T,
        seg: u16,
        addr: u16,
        value: u16,
    ) {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xf_ffff;
        ctx.mem_write_byte(masked_addr, value as u8);
        ctx.mem_write_byte(masked_addr + 1, (value >> 8) as u8);
    }

    pub fn set_parity_flag(&mut self, mut data: u16) {
        let mut parity = 0;
        while data != 0 {
            parity ^= data & 1;
            data >>= 1;
        }
        self.regs.flags.set(Flags::PARITY, parity != 0);
    }

    pub fn set_pzs8(&mut self, data: u8) {
        self.set_parity_flag(data as u16);
        self.regs.flags.set(Flags::ZERO, data == 0);
        self.regs.flags.set(Flags::SIGN, (data & 0x80) == 0x80);
    }

    pub fn set_pzs16(&mut self, data: u16) {
        self.set_parity_flag(data);
        self.regs.flags.set(Flags::ZERO, data == 0);
        self.regs.flags.set(Flags::SIGN, (data & 0x8000) == 0x8000);
    }

    pub fn pop16<T: Cpu8086Context>(&mut self, ctx: &mut T) -> u16 {
        let stack_pointer = self.regs.read16(Reg16::SP);
        self.regs.write16(Reg16::SP, stack_pointer.wrapping_add(2));
        self.mem_read_word(ctx, self.regs.readseg16(SegReg::SS), stack_pointer)
    }

    pub fn tick<T: Cpu8086Context>(&mut self, ctx: &mut T) -> usize {
        self.opcode = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip);
        println!(
            "Opcode {:#02x} CS {:#04x} IP {:#04x}\nGPRs {:x?} Segments {:x?}\nFLAGS {:#04x}",
            self.opcode,
            self.regs.readseg16(SegReg::CS),
            self.regs.ip,
            self.regs.gprs,
            self.regs.seg_regs,
            self.regs.flags.bits()
        );
        match self.opcode {
            0x00 => {
                println!("add rm8, reg8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                let mut segment_long: SegReg = SegReg::DS;
                let mut opcode_rm_long: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                    opcode_rm_long = opcode_rm;
                    segment_long = segment;
                }
                let result = reg.wrapping_add(rm);
                self.set_pzs8(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x80) > (result & 0x80));
                self.mem_write_byte(
                    ctx,
                    self.regs.readseg16(segment_long),
                    opcode_rm_long,
                    result,
                );
            }
            0x02 => {
                println!("add reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg.wrapping_add(rm);
                self.set_pzs8(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x80) > (result & 0x80));
                self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
            }
            0x03 => {
                println!("add reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg.wrapping_add(rm);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x8000) > (result & 0x8000));
                self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
            }
            0x06 => {
                println!("push es");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.readseg16(SegReg::ES),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x07 => {
                println!("pop es");
                let es = self.pop16(ctx);
                self.regs.writeseg16(SegReg::ES, es);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x0a => {
                println!("or reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg | rm;
                self.set_pzs8(result);
                self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
            }
            0x0b => {
                println!("or reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm_reg) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm_reg).unwrap());
                } else if let Operand::Address(segment, opcode_rm_addr) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm_addr);
                }
                let result = reg | rm;
                self.set_pzs16(result);
                self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
            }
            0x13 => {
                println!("adc reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let carry: u16 = if self.regs.flags.contains(Flags::CARRY) { 1 } else { 0 };
                let result = reg.wrapping_add(rm).wrapping_add(carry);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x8000) > (result & 0x8000));
                self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
            }
            0x16 => {
                println!("push ss");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.readseg16(SegReg::SS),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x1e => {
                println!("push ds");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.readseg16(SegReg::DS),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x1f => {
                println!("pop ds");
                let ds = self.pop16(ctx);
                self.regs.writeseg16(SegReg::DS, ds);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x22 => {
                println!("and reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg | rm;
                self.set_pzs8(result);
                self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
            }
            0x23 => {
                println!("and reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg & rm;
                self.set_pzs16(result);
                self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
            }
            0x24 => {
                println!("and al, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::AL, self.regs.read8(Reg8::AL) & imm_value);
                self.set_pzs8(self.regs.read8(Reg8::AL));
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0x26 => {
                println!("es:");
                self.seg_override = Some(SegReg::ES);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0x2a => {
                println!("sub reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg.wrapping_sub(rm);
                self.set_pzs8(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x80) > (reg & 0x80));
                self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
            }
            0x2b => {
                println!("sub reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg.wrapping_sub(rm);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ rm) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x8000) > (reg & 0x8000));
                self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
            }
            0x2e => {
                println!("cs:");
                self.seg_override = Some(SegReg::CS);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0x32 => {
                println!("xor reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg ^ rm;
                self.set_pzs8(result);
                self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
            }
            0x33 => {
                println!("xor reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg ^ rm;
                self.set_pzs16(result);
                self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
            }
            0x36 => {
                println!("ss:");
                self.seg_override = Some(SegReg::SS);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0x39 => {
                println!("cmp rm16, reg16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                let mut segment_long: SegReg = SegReg::DS;
                let mut opcode_rm_long: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                    opcode_rm_long = opcode_rm;
                    segment_long = segment;
                }
                let result = reg.wrapping_sub(rm);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x80) > (result & 0x80));
            }
            0x3a => {
                println!("cmp reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                let mut rm: u8 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg.wrapping_sub(rm);
                self.set_pzs8(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ rm) & (result ^ reg) & 0x80) == 0x80,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x80) > (reg & 0x80));
            }
            0x3b => {
                println!("cmp reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                let reg = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                let mut rm: u16 = 0;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                }
                let result = reg.wrapping_sub(rm);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ rm) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                self.regs
                    .flags
                    .set(Flags::CARRY, (rm & 0x8000) > (reg & 0x8000));
            }
            0x3e => {
                println!("ds:");
                self.seg_override = Some(SegReg::DS);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0x40 => {
                println!("inc ax");
                let reg: u16 = self.regs.read16(Reg16::AX);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::AX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x41 => {
                println!("inc cx");
                let reg: u16 = self.regs.read16(Reg16::CX);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::CX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x42 => {
                println!("inc dx");
                let reg: u16 = self.regs.read16(Reg16::DX);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::DX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x43 => {
                println!("inc bx");
                let reg: u16 = self.regs.read16(Reg16::BX);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::BX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x44 => {
                println!("inc sp");
                let reg: u16 = self.regs.read16(Reg16::SP);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::SP, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x45 => {
                println!("inc bp");
                let reg: u16 = self.regs.read16(Reg16::BP);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::BP, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x46 => {
                println!("inc si");
                let reg: u16 = self.regs.read16(Reg16::SI);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::SI, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x47 => {
                println!("inc di");
                let reg: u16 = self.regs.read16(Reg16::DI);
                let result = reg.wrapping_add(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::DI, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x48 => {
                println!("dec ax");
                let reg: u16 = self.regs.read16(Reg16::AX);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::AX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x49 => {
                println!("dec cx");
                let reg: u16 = self.regs.read16(Reg16::CX);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::CX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x4a => {
                println!("dec dx");
                let reg: u16 = self.regs.read16(Reg16::DX);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::DX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x4b => {
                println!("dec bx");
                let reg: u16 = self.regs.read16(Reg16::BX);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::BX, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x4c => {
                println!("dec sp");
                let reg: u16 = self.regs.read16(Reg16::SP);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::SP, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x4d => {
                println!("dec bp");
                let reg: u16 = self.regs.read16(Reg16::BP);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::BP, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x4e => {
                println!("dec si");
                let reg: u16 = self.regs.read16(Reg16::SI);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::SI, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x4f => {
                println!("dec di");
                let reg: u16 = self.regs.read16(Reg16::DI);
                let result = reg.wrapping_sub(1);
                self.set_pzs16(result);
                self.regs.flags.set(
                    Flags::OVERFLOW,
                    ((reg ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                );
                self.regs
                    .flags
                    .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                self.regs.write16(Reg16::DI, result);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x50 => {
                println!("push ax");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::AX),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x51 => {
                println!("push cx");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::CX),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x52 => {
                println!("push dx");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::DX),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x53 => {
                println!("push bx");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::BX),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x54 => {
                println!("push sp");
                let sp = self.regs.read16(Reg16::SP);
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    sp,
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x55 => {
                println!("push bp");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::BP),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x56 => {
                println!("push si");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::SI),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x57 => {
                println!("push di");
                self.regs
                    .write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(
                    ctx,
                    self.regs.readseg16(SegReg::SS),
                    self.regs.read16(Reg16::SP),
                    self.regs.read16(Reg16::DI),
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x58 => {
                println!("pop ax");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::AX, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x59 => {
                println!("pop cx");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::CX, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x5a => {
                println!("pop dx");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::DX, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x5b => {
                println!("pop bx");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::BX, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x5c => {
                println!("pop sp");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::SP, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x5d => {
                println!("pop bp");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::BP, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x5e => {
                println!("pop si");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::SI, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x5f => {
                println!("pop di");
                let tmp = self.pop16(ctx);
                self.regs.write16(Reg16::DI, tmp);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0x70 => {
                println!("jo");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if self.regs.flags.contains(Flags::OVERFLOW) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x71 => {
                println!("jno");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if !self.regs.flags.contains(Flags::OVERFLOW) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x72 => {
                println!("jc");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if self.regs.flags.contains(Flags::CARRY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x73 => {
                println!("jnc");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if !self.regs.flags.contains(Flags::CARRY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x74 => {
                println!("jz");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if self.regs.flags.contains(Flags::ZERO) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x75 => {
                println!("jnz");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if !self.regs.flags.contains(Flags::ZERO) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x78 => {
                println!("js");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if self.regs.flags.contains(Flags::SIGN) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x79 => {
                println!("jns");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if !self.regs.flags.contains(Flags::SIGN) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x7a => {
                println!("jp");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if self.regs.flags.contains(Flags::PARITY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x7b => {
                println!("jnp");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if !self.regs.flags.contains(Flags::PARITY) {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0x80 => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let imm: u8 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip,
                );
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    1 => {
                        println!("or rm8, imm8");
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            let result = reg | imm;
                            self.set_pzs8(result);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
                        }
                        if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                            let src = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                            let result = src | imm;
                            self.set_pzs8(result);
                            self.mem_write_byte(ctx, self.regs.readseg16(segment), opcode_rm, result);
                        }
                    }
                    7 => {
                        println!("cmp rm8, imm8");
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            let result = reg.wrapping_sub(imm);
                            self.set_pzs8(result);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                ((reg ^ imm) & (result ^ reg) & 0x80) == 0x80,
                            );
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ reg ^ imm) & 0x10) == 0x10);
                            self.regs
                                .flags
                                .set(Flags::CARRY, (imm & 0x80) > (reg & 0x80));
                        }
                        if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                            let src = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                            let result = src.wrapping_sub(imm);
                            self.set_pzs8(result);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                ((src ^ imm) & (result ^ src) & 0x80) == 0x80,
                            );
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ src ^ imm) & 0x10) == 0x10);
                            self.regs
                                .flags
                                .set(Flags::CARRY, (imm & 0x80) > (src & 0x80));
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            0x88 => {
                println!("mov rm8, reg8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    self.regs.write8(
                        Reg8::from_num(opcode_rm).unwrap(),
                        self.regs.read8(Reg8::from_num(reg_num).unwrap()),
                    );
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    self.mem_write_byte(
                        ctx,
                        self.regs.readseg16(segment),
                        opcode_rm,
                        self.regs.read8(Reg8::from_num(reg_num).unwrap()),
                    );
                }
            }
            0x89 => {
                println!("mov rm16, reg16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    self.regs.write16(
                        Reg16::from_num(opcode_rm).unwrap(),
                        self.regs.read16(Reg16::from_num(reg_num).unwrap()),
                    );
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    self.mem_write_word(
                        ctx,
                        self.regs.readseg16(segment),
                        opcode_rm,
                        self.regs.read16(Reg16::from_num(reg_num).unwrap()),
                    );
                }
            }
            0x8a => {
                println!("mov reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    self.regs.write8(
                        Reg8::from_num(reg_num).unwrap(),
                        self.regs.read8(Reg8::from_num(opcode_rm).unwrap()),
                    );
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    let rm = self.mem_read_byte(ctx, self.regs.readseg16(segment), opcode_rm);
                    self.regs.write8(Reg8::from_num(reg_num).unwrap(), rm);
                }
            }
            0x8b => {
                println!("mov reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    self.regs.write16(
                        Reg16::from_num(reg_num).unwrap(),
                        self.regs.read16(Reg16::from_num(opcode_rm).unwrap()),
                    );
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    let rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                    self.regs.write16(Reg16::from_num(reg_num).unwrap(), rm);
                }
            }
            0x8c => {
                println!("mov rm, seg");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    self.regs.write16(
                        Reg16::from_num(opcode_rm).unwrap(),
                        self.regs.readseg16(SegReg::from_num(reg_num).unwrap()),
                    );
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    self.mem_write_word(
                        ctx,
                        self.regs.readseg16(segment),
                        opcode_rm,
                        self.regs.readseg16(SegReg::from_num(reg_num).unwrap()),
                    );
                }
            }
            0x8e => {
                println!("mov seg, rm");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg_num = (modrm & 0x38) >> 3;
                if let Operand::Register(opcode_rm) = opcode_params.rm {
                    self.regs.writeseg16(
                        SegReg::from_num(reg_num).unwrap(),
                        self.regs.read16(Reg16::from_num(opcode_rm).unwrap()),
                    );
                } else if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    let rm = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                    self.regs.writeseg16(SegReg::from_num(reg_num).unwrap(), rm);
                }
            }
            0x96 => {
                println!("xchg si,ax");
                self.regs.ip = self.regs.ip.wrapping_add(1);
                let oldsi = self.regs.read16(Reg16::SI);
                let oldax = self.regs.read16(Reg16::AX);
                self.regs.write16(Reg16::SI, oldax);
                self.regs.write16(Reg16::AX, oldsi);
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
                self.regs
                    .write8(Reg8::AH, (self.regs.flags.bits() & 0xd5) as u8);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xa0 => {
                println!("mov al, [imm]");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(3);
                let mut result: u8 = 0;
                if let Some(segment) = self.seg_override {
                    result = self.mem_read_byte(ctx, self.regs.readseg16(segment), imm_value);
                }
                else {
                    result = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), imm_value);
                }
                self.regs.write8(Reg8::AL, result);
            }
            0xa1 => {
                println!("mov ax, [imm]");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(3);
                let mut result: u16 = 0;
                if let Some(segment) = self.seg_override {
                    result = self.mem_read_word(ctx, self.regs.readseg16(segment), imm_value);
                }
                else {
                    result = self.mem_read_word(ctx, self.regs.readseg16(SegReg::DS), imm_value);
                }
                self.regs.write16(Reg16::AX, result);
            }
            0xa4 => {
                println!("movsb");
                if self.rep_state == None {
                    let data = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                    self.mem_write_byte(ctx, self.regs.readseg16(SegReg::ES), self.regs.read16(Reg16::DI), data);
                    let mut offset: i16 = 1;
                    if self.regs.flags.contains(Flags::DIRECTION) {
                        offset = -1;
                    }
                    self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                    self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                }
                else {
                    loop {
                        let data = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                        self.mem_write_byte(ctx, self.regs.readseg16(SegReg::ES), self.regs.read16(Reg16::DI), data);
                        let mut offset: i16 = 1;
                        if self.regs.flags.contains(Flags::DIRECTION) {
                            offset = -1;
                        }
                        self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                        self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                        self.regs.write16(Reg16::CX, self.regs.read16(Reg16::CX).wrapping_sub(1));
                        if self.regs.read16(Reg16::CX) == 0 { break; }
                    }
                }
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xa6 => {
                println!("cmpsb");
                match self.rep_state {
                    None => {
                        let src = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                        let dst = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::ES), self.regs.read16(Reg16::DI));
                        let result = dst.wrapping_sub(src);
                        let mut offset: i16 = 1;
                        self.set_pzs8(result);
                        self.regs.flags.set(
                            Flags::OVERFLOW,
                            ((result ^ src) & (result ^ dst) & 0x80) == 0x80,
                        );
                        self.regs
                            .flags
                            .set(Flags::ADJUST, ((result ^ src ^ dst) & 0x10) == 0x10);
                        self.regs
                            .flags
                            .set(Flags::CARRY, (dst & 0x80) > (result & 0x80));
                        if self.regs.flags.contains(Flags::DIRECTION) {
                            offset = -1;
                        }
                        self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                        self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                    }   
                    Some(RepType::REPE) => {
                        loop {
                            let src = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                            let dst = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::ES), self.regs.read16(Reg16::DI));
                            let result = dst.wrapping_sub(src);
                            let mut offset: i16 = 1;
                            self.set_pzs8(result);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                ((result ^ src) & (result ^ dst) & 0x80) == 0x80,
                            );
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ src ^ dst) & 0x10) == 0x10);
                            self.regs
                                .flags
                                .set(Flags::CARRY, (dst & 0x80) > (result & 0x80));
                            if self.regs.flags.contains(Flags::DIRECTION) {
                                offset = -1;
                            }
                            self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                            self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                            self.regs.write16(Reg16::CX, self.regs.read16(Reg16::CX).wrapping_sub(1));
                            if self.regs.read16(Reg16::CX) == 0 { break; }
                            if self.regs.flags.contains(Flags::ZERO) { break; }
                        }
                    }
                    Some(RepType::REPNE) => {
                        loop {
                            let src = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                            let dst = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::ES), self.regs.read16(Reg16::DI));
                            let result = dst.wrapping_sub(src);
                            let mut offset: i16 = 1;
                            self.set_pzs8(result);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                ((result ^ src) & (result ^ dst) & 0x80) == 0x80,
                            );
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ src ^ dst) & 0x10) == 0x10);
                            self.regs
                                .flags
                                .set(Flags::CARRY, (dst & 0x80) > (result & 0x80));
                            if self.regs.flags.contains(Flags::DIRECTION) {
                                offset = -1;
                            }
                            self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                            self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                            self.regs.write16(Reg16::CX, self.regs.read16(Reg16::CX).wrapping_sub(1));
                            if self.regs.read16(Reg16::CX) == 0 { break; }
                            if !self.regs.flags.contains(Flags::ZERO) { break; }
                        }
                    }
                }
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xac => {
                println!("lodsb");
                if self.rep_state == None {
                    let data = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                    self.regs.write8(Reg8::AL, data);
                    let mut offset: i16 = 1;
                    if self.regs.flags.contains(Flags::DIRECTION) {
                        offset = -1;
                    }
                    self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                    self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                }
                else {
                    loop {
                        let data = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::DS), self.regs.read16(Reg16::SI));
                        self.regs.write8(Reg8::AL, data);
                        let mut offset: i16 = 1;
                        if self.regs.flags.contains(Flags::DIRECTION) {
                            offset = -1;
                        }
                        self.regs.write16(Reg16::SI, self.regs.read16(Reg16::SI).wrapping_add(offset as u16));
                        self.regs.write16(Reg16::DI, self.regs.read16(Reg16::DI).wrapping_add(offset as u16));
                        self.regs.write16(Reg16::CX, self.regs.read16(Reg16::CX).wrapping_sub(1));
                        if self.regs.read16(Reg16::CX) == 0 { break; }
                    }
                }
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xb0 => {
                println!("mov al, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::AL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb1 => {
                println!("mov cl, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::CL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb2 => {
                println!("mov dl, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::DL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb3 => {
                println!("mov bl, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::BL, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb4 => {
                println!("mov ah, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::AH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb5 => {
                println!("mov ch, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::CH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb6 => {
                println!("mov dh, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::DH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb7 => {
                println!("mov bh, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write8(Reg8::BH, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xb8 => {
                println!("mov ax, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::AX, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xb9 => {
                println!("mov cx, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::CX, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xba => {
                println!("mov dx, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::DX, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xbb => {
                println!("mov bx, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::BX, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xbc => {
                println!("mov sp, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::SP, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xbd => {
                println!("mov bp, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::BP, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xbe => {
                println!("mov si, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::SI, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xbf => {
                println!("mov di, imm");
                let imm_value = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.write16(Reg16::DI, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xc3 => {
                println!("ret");
                self.regs.ip = self.pop16(ctx);
            }
            0xc4 => {
                println!("les");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg = (modrm & 0x38) >> 3;
                if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    let addr = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                    let seg = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm + 2);
                    self.regs.writeseg16(SegReg::ES, seg);
                    self.regs.write16(Reg16::from_num(reg).unwrap(), addr);
                }
            }
            0xc5 => {
                println!("lds");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let reg = (modrm & 0x38) >> 3;
                if let Operand::Address(segment, opcode_rm) = opcode_params.rm {
                    let addr = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm);
                    let seg = self.mem_read_word(ctx, self.regs.readseg16(segment), opcode_rm + 2);
                    self.regs.writeseg16(SegReg::DS, seg);
                    self.regs.write16(Reg16::from_num(reg).unwrap(), addr);
                }
            }
            0xc6 => {
                println!("mov rm8, imm");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                match opcode_params.rm {
                    Operand::Address(_, _) => (),
                    _ => panic!("Opcode doesn't support register operands!"),
                }
                let imm = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip.wrapping_add(1));
                if let Operand::Address(easeg, ea) = opcode_params.rm {
                    self.mem_write_byte(ctx, self.regs.readseg16(easeg), ea, imm);
                }
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xc7 => {
                println!("mov rm16, imm");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                match opcode_params.rm {
                    Operand::Address(_, _) => (),
                    _ => panic!("Opcode doesn't support register operands!"),
                }
                let imm = self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip.wrapping_add(1));
                if let Operand::Address(easeg, ea) = opcode_params.rm {
                    self.mem_write_word(ctx, self.regs.readseg16(easeg), ea, imm);
                }
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xcd => {
                let intr = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                println!("int {:x}", intr);
                self.interrupt_hook(ctx, intr);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xd0 => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Opcode doesn't support memory operands!"),
                }
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    4 => {
                        println!("shl reg, 1");
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            self.regs.flags.set(Flags::CARRY, (reg & 1) == 1);
                            let overflow_calc = ((reg >> 7) & 1) ^ ((reg >> 6) & 1);
                            self.regs.flags.set(Flags::OVERFLOW, overflow_calc == 1);
                            reg = reg.wrapping_shl(1);
                            self.set_pzs8(reg);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), reg);
                        }
                    }
                    5 => {
                        println!("shr reg, 1");
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            self.regs.flags.set(Flags::CARRY, (reg & 1) == 1);
                            self.regs.flags.set(Flags::OVERFLOW, (reg & 0x80) == 0x80);
                            reg = reg.wrapping_shr(1);
                            self.set_pzs8(reg);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), reg);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            0xd2 => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Opcode doesn't support memory operands!"),
                }
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    4 => {
                        println!("shl reg, cl");
                        let mut count = self.regs.read8(Reg8::CL);
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            while count != 0 {
                                self.regs.flags.set(Flags::CARRY, (reg & 0x80) == 0x80);
                                reg = reg.wrapping_shl(1);
                                count = count.wrapping_sub(1);
                            }
                            self.set_pzs8(reg);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), reg);
                        }
                    }
                    5 => {
                        println!("shr reg, cl");
                        let mut count = self.regs.read8(Reg8::CL);
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            while count != 0 {
                                self.regs.flags.set(Flags::CARRY, (reg & 1) == 1);
                                reg = reg.wrapping_shr(1);
                                count = count.wrapping_sub(1);
                            }
                            self.set_pzs8(reg);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), reg);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            0xe2 => {
                println!("loop");
                let offset: i16 = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                ) as i8 as i16;
                self.regs
                    .write16(Reg16::CX, self.regs.read16(Reg16::CX).wrapping_sub(1));
                self.regs.ip = self.regs.ip.wrapping_add(2);
                if self.regs.read16(Reg16::CX) != 0 {
                    self.regs.ip = self.regs.ip.wrapping_add(offset as u16);
                }
            }
            0xe4 => {
                println!("in al, imm");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                let result = self.io_read_byte(ctx, imm_value as u16);
                self.regs.write8(Reg8::AL, result);
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xe6 => {
                println!("out imm, al");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.io_write_byte(ctx, imm_value as u16, self.regs.read8(Reg8::AL));
                self.regs.ip = self.regs.ip.wrapping_add(2);
            }
            0xe8 => {
                println!("call near");
                let offset = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(3);
                self.regs.write16(Reg16::SP, self.regs.read16(Reg16::SP).wrapping_sub(2));
                self.mem_write_word(ctx, self.regs.readseg16(SegReg::SS), self.regs.read16(Reg16::SP), self.regs.ip);
                self.regs.ip = self.regs.ip.wrapping_add(offset);
            }
            0xe9 => {
                println!("jmp near");
                let offset = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(offset + 3);
            }
            0xea => {
                println!("jmp far");
                let offset = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                let segment = self.mem_read_word(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(3),
                );
                self.regs.writeseg16(SegReg::CS, segment);
                self.regs.ip = offset;
            }
            0xeb => {
                println!("jmp rel8");
                let offset = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add((offset as i8 as u16) + 2u16);
            }
            0xee => {
                println!("out dx, al");
                self.io_write_byte(ctx, self.regs.read16(Reg16::DX), self.regs.read8(Reg8::AL));
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xf2 => {
                println!("repne:");
                self.rep_state = Some(RepType::REPNE);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0xf3 => {
                println!("repe:");
                self.rep_state = Some(RepType::REPE);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0xf7 => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    4 => {
                        println!("mul rm16");
                        if let Operand::Address(easeg, ea) = opcode_params.rm {
                            let reg: u16 = self.regs.read16(Reg16::AX);
                            let dst: u16 = self.mem_read_word(ctx, self.regs.readseg16(easeg), ea);
                            let result: u32 = (reg as u32) * (dst as u32);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                (result & 0xffff0000u32) != 0,
                            );
                            self.regs
                                .flags
                                .set(Flags::CARRY, (result & 0xffff0000u32) != 0);
                            self.regs.write16(Reg16::AX, result as u16);
                            self.regs.write16(Reg16::DX, (result >> 16) as u16);
                        }
                        else if let Operand::Register(rm) = opcode_params.rm {
                            let reg: u16 = self.regs.read16(Reg16::AX);
                            let dst: u16 = self.regs.read16(Reg16::from_num(rm).unwrap());
                            let result: u32 = (reg as u32) * (dst as u32);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                (result & 0xffff0000u32) != 0,
                            );
                            self.regs
                                .flags
                                .set(Flags::CARRY, (result & 0xffff0000u32) != 0);
                            self.regs.write16(Reg16::AX, result as u16);
                            self.regs.write16(Reg16::DX, (result >> 16) as u16);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            0xf8 => {
                println!("clc");
                self.regs.flags.set(Flags::CARRY, false);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xf9 => {
                println!("stc");
                self.regs.flags.set(Flags::CARRY, true);
                self.regs.ip = self.regs.ip.wrapping_add(1);
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
            0xfc => {
                println!("cld");
                self.regs.flags.set(Flags::DIRECTION, false);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xfd => {
                println!("std");
                self.regs.flags.set(Flags::DIRECTION, true);
                self.regs.ip = self.regs.ip.wrapping_add(1);
            }
            0xfe => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    0 => {
                        println!("inc rm8");
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            let result = reg.wrapping_add(1);
                            self.set_pzs8(result);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                ((result ^ 1) & (result ^ reg) & 0x80) == 0x80,
                            );
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
                        }
                    }
                    1 => {
                        println!("dec rm8");
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let reg: u8 = self.regs.read8(Reg8::from_num(reg_num).unwrap());
                            let result = reg.wrapping_sub(1);
                            self.set_pzs8(result);
                            self.regs
                                .flags
                                .set(Flags::OVERFLOW, ((reg ^ 1) & (result ^ reg) & 0x80) == 0x80);
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                            self.regs.write8(Reg8::from_num(reg_num).unwrap(), result);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            0xff => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(ctx, modrm);
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    0 => {
                        println!("inc rm16");
                        match opcode_params.rm {
                            Operand::Register(_) => (),
                            _ => panic!("Memory operands not supported yet!"),
                        }
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let reg: u16 = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                            let result = reg.wrapping_add(1);
                            self.set_pzs16(result);
                            self.regs.flags.set(
                                Flags::OVERFLOW,
                                ((result ^ 1) & (result ^ reg) & 0x8000) == 0x8000,
                            );
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                            self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
                        }
                    }
                    1 => {
                        println!("dec rm16");
                        match opcode_params.rm {
                            Operand::Register(_) => (),
                            _ => panic!("Memory operands not supported yet!"),
                        }
                        if let Operand::Register(reg_num) = opcode_params.rm {
                            let reg: u16 = self.regs.read16(Reg16::from_num(reg_num).unwrap());
                            let result = reg.wrapping_sub(1);
                            self.set_pzs16(result);
                            self.regs
                                .flags
                                .set(Flags::OVERFLOW, ((reg ^ 1) & (result ^ reg) & 0x80) == 0x80);
                            self.regs
                                .flags
                                .set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                            self.regs.write16(Reg16::from_num(reg_num).unwrap(), result);
                        }
                    }
                    5 => {
                        println!("jmp far");
                        match opcode_params.rm {
                            Operand::Address(_, _) => (),
                            _ => panic!("Register operands not supported yet!"),
                        }
                        if let Operand::Address(easeg, ea) = opcode_params.rm {
                            let offset = self.mem_read_word(ctx, self.regs.readseg16(easeg), ea);
                            let segment = self.mem_read_word(ctx, self.regs.readseg16(easeg), ea.wrapping_add(2));
                            self.regs.ip = offset;
                            self.regs.writeseg16(SegReg::CS, segment);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            _ => panic!("Unhandled opcode!"),
        }
        self.seg_override = None;
        4
    }
}
