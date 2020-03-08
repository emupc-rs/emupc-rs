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

#[derive(Clone, Copy, Debug)]
pub struct Cpu8086 {
    pub regs: Registers,
    pub opcode: u8,
    pub seg_override: Option<SegReg>,
}

impl Cpu8086 {
    pub fn new() -> Cpu8086 {
        Cpu8086 {
            regs: Registers::new(),
            opcode: 0,
            seg_override: None,
        }
    }
    pub fn mem_read_byte<T: Cpu8086Context>(&mut self, ctx: &mut T, seg: u16, addr: u16) -> u8 {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xfffff;
        ctx.mem_read_byte(masked_addr)
    }
    pub fn mem_write_byte<T: Cpu8086Context>(
        &mut self,
        ctx: &mut T,
        seg: u16,
        addr: u16,
        value: u8,
    ) {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xfffff;
        ctx.mem_write_byte(masked_addr, value)
    }

    pub fn io_read_byte<T: Cpu8086Context>(&mut self, ctx: &mut T, addr: u16) -> u8 {
        ctx.io_read_byte(addr)
    }

    pub fn io_write_byte<T: Cpu8086Context>(&mut self, ctx: &mut T, addr: u16, value: u8) {
        ctx.io_write_byte(addr, value)
    }

    pub fn mem_read_word<T: Cpu8086Context>(&mut self, ctx: &mut T, seg: u16, addr: u16) -> u16 {
        let masked_addr = (((seg as u32) << 4) | addr as u32) & 0xfffff;
        let lo = ctx.mem_read_byte(masked_addr);
        let hi = ctx.mem_read_byte(masked_addr.wrapping_add(1) & 0xfffff);
        u16::from_le_bytes([lo, hi])
    }

    pub fn set_parity_flag(&mut self, mut data: u16) {
        let mut parity = 0;
        while data != 0 {
            parity ^= data & 1;
            data = data >> 1;
        }
        self.regs.flags.set(Flags::PARITY, parity != 0);
    }

    pub fn tick<T: Cpu8086Context>(&mut self, ctx: &mut T) {
        self.opcode = self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip);
        println!(
            "Opcode {:#02x} CS {:#04x} IP {:#04x}\nGPRs {:x?} FLAGS {:#04x}",
            self.opcode,
            self.regs.readseg16(SegReg::CS),
            self.regs.ip,
            self.regs.gprs,
            self.regs.flags.bits()
        );
        match self.opcode {
            0x02 => {
                println!("add reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let reg = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                        let rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                        let result = reg - rm;
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                        self.set_parity_flag(result as u16);
                        self.regs.flags.set(Flags::OVERFLOW, ((result ^ rm) & (result ^ reg) & 0x80) == 0x80);
                        self.regs.flags.set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                        self.regs
                            .write8(Reg8::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x03 => {
                println!("add reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let reg = self.regs.read16(Reg16::from_num(opcode_reg).unwrap());
                        let rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                        let result = reg - rm;
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x8000) == 0x8000);
                        self.set_parity_flag(result);
                        self.regs.flags.set(Flags::OVERFLOW, ((result ^ rm) & (result ^ reg) & 0x8000) == 0x8000);
                        self.regs.flags.set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                        self.regs
                            .write16(Reg16::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x0a => {
                println!("or reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let result = self.regs.read8(Reg8::from_num(opcode_reg).unwrap())
                            | self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                        self.set_parity_flag(result as u16);
                        self.regs
                            .write8(Reg8::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x0b => {
                println!("or reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let result = self.regs.read16(Reg16::from_num(opcode_reg).unwrap())
                            | self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x8000) == 0x8000);
                        self.set_parity_flag(result);
                        self.regs
                            .write16(Reg16::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x22 => {
                println!("and reg8, rm8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let result = self.regs.read8(Reg8::from_num(opcode_reg).unwrap())
                            & self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                        self.set_parity_flag(result as u16);
                        self.regs
                            .write8(Reg8::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x23 => {
                println!("and reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let result = self.regs.read16(Reg16::from_num(opcode_reg).unwrap())
                            & self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x8000) == 0x8000);
                        self.set_parity_flag(result);
                        self.regs
                            .write16(Reg16::from_num(opcode_reg).unwrap(), result);
                    }
                }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let reg = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                        let rm = self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                        let result = reg - rm;
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                        self.set_parity_flag(result as u16);
                        self.regs.flags.set(Flags::OVERFLOW, ((reg ^ rm) & (result ^ reg) & 0x80) == 0x80);
                        self.regs.flags.set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                        self.regs
                            .write8(Reg8::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x2b => {
                println!("sub reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let reg = self.regs.read16(Reg16::from_num(opcode_reg).unwrap());
                        let rm = self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                        let result = reg - rm;
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x8000) == 0x8000);
                        self.set_parity_flag(result);
                        self.regs.flags.set(Flags::OVERFLOW, ((reg ^ rm) & (result ^ reg) & 0x8000) == 0x8000);
                        self.regs.flags.set(Flags::ADJUST, ((result ^ reg ^ rm) & 0x10) == 0x10);
                        self.regs
                            .write16(Reg16::from_num(opcode_reg).unwrap(), result);
                    }
                }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let result = self.regs.read8(Reg8::from_num(opcode_reg).unwrap())
                            ^ self.regs.read8(Reg8::from_num(opcode_rm).unwrap());
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                        self.set_parity_flag(result as u16);
                        self.regs
                            .write8(Reg8::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x33 => {
                println!("xor reg16, rm16");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                self.regs.flags.set(Flags::OVERFLOW, false);
                self.regs.flags.set(Flags::CARRY, false);
                //A bit ugly, but I can't figure out any other way to do this
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        let result = self.regs.read16(Reg16::from_num(opcode_reg).unwrap())
                            ^ self.regs.read16(Reg16::from_num(opcode_rm).unwrap());
                        self.regs.flags.set(Flags::ZERO, result == 0);
                        self.regs.flags.set(Flags::SIGN, (result & 0x8000) == 0x8000);
                        self.set_parity_flag(result);
                        self.regs
                            .write16(Reg16::from_num(opcode_reg).unwrap(), result);
                    }
                }
            }
            0x36 => {
                println!("ss:");
                self.seg_override = Some(SegReg::SS);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
            }
            0x3e => {
                println!("ds:");
                self.seg_override = Some(SegReg::DS);
                self.regs.ip = self.regs.ip.wrapping_add(1);
                self.tick(ctx);
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
            0x88 => {
                println!("mov rm8, reg8");
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        self.regs.write8(
                            Reg8::from_num(opcode_rm).unwrap(),
                            self.regs.read8(Reg8::from_num(opcode_reg).unwrap()),
                        );
                    }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        self.regs.write16(
                            Reg16::from_num(opcode_rm).unwrap(),
                            self.regs.read16(Reg16::from_num(opcode_reg).unwrap()),
                        );
                    }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        self.regs.write8(
                            Reg8::from_num(opcode_reg).unwrap(),
                            self.regs.read8(Reg8::from_num(opcode_rm).unwrap()),
                        );
                    }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        self.regs.write16(
                            Reg16::from_num(opcode_reg).unwrap(),
                            self.regs.read16(Reg16::from_num(opcode_rm).unwrap()),
                        );
                    }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        self.regs.write16(
                            Reg16::from_num(opcode_rm).unwrap(),
                            self.regs.readseg16(SegReg::from_num(opcode_reg).unwrap()),
                        );
                    }
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                if let Operand::Register(opcode_reg) = opcode_params.reg {
                    if let Operand::Register(opcode_rm) = opcode_params.rm {
                        self.regs.writeseg16(
                            SegReg::from_num(opcode_reg).unwrap(),
                            self.regs.read16(Reg16::from_num(opcode_rm).unwrap()),
                        );
                    }
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
                self.regs
                    .write8(Reg8::AH, (self.regs.flags.bits() & 0xd5) as u8);
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
                self.regs.write16(Reg16::SI, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(3);
            }
            0xd0 => {
                let modrm = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.regs.ip = self.regs.ip.wrapping_add(2);
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Opcode doesn't support memory operands!"),
                }
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    4 => {
                        println!("shl reg, 1");
                        if let Operand::Register(opcode_reg) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                            self.regs.flags.set(Flags::CARRY, (reg & 1) == 1);
                            let overflow_calc = ((reg >> 7) & 1) ^ ((reg >> 6) & 1);
                            self.regs.flags.set(Flags::OVERFLOW, overflow_calc == 1);
                            reg = reg.wrapping_shl(1);
                            self.regs.write8(Reg8::from_num(opcode_reg).unwrap(), reg);
                        }
                    }
                    5 => {
                        println!("shr reg, 1");
                        if let Operand::Register(opcode_reg) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                            self.regs.flags.set(Flags::CARRY, (reg & 1) == 1);
                            self.regs.flags.set(Flags::OVERFLOW, (reg & 0x80) == 0x80);
                            reg = reg.wrapping_shr(1);
                            self.regs.write8(Reg8::from_num(opcode_reg).unwrap(), reg);
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Opcode doesn't support memory operands!"),
                }
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    4 => {
                        println!("shl reg, cl");
                        let mut count = self.regs.read8(Reg8::CL);
                        if let Operand::Register(opcode_reg) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                            while count != 0 {
                                self.regs.flags.set(Flags::CARRY, (reg & 0x80) == 0x80);
                                reg = reg.wrapping_shl(1);
                                count = count.wrapping_sub(1);
                            }
                            self.regs.write8(Reg8::from_num(opcode_reg).unwrap(), reg);
                        }
                    }
                    5 => {
                        println!("shr reg, cl");
                        let mut count = self.regs.read8(Reg8::CL);
                        if let Operand::Register(opcode_reg) = opcode_params.rm {
                            let mut reg: u8 = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                            while count != 0 {
                                self.regs.flags.set(Flags::CARRY, (reg & 1) == 1);
                                reg = reg.wrapping_shr(1);
                                count = count.wrapping_sub(1);
                            }
                            self.regs.write8(Reg8::from_num(opcode_reg).unwrap(), reg);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            0xe6 => {
                println!("out imm, al");
                let imm_value = self.mem_read_byte(
                    ctx,
                    self.regs.readseg16(SegReg::CS),
                    self.regs.ip.wrapping_add(1),
                );
                self.io_write_byte(ctx, self.regs.read8(Reg8::AL) as u16, imm_value);
                self.regs.ip = self.regs.ip.wrapping_add(2);
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
            0xee => {
                println!("out dx, al");
                self.io_write_byte(ctx, self.regs.read16(Reg16::DX), self.regs.read8(Reg8::AL));
                self.regs.ip = self.regs.ip.wrapping_add(1);
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
                let opcode_params = self.get_opcode_params_from_modrm(modrm);
                match opcode_params.rm {
                    Operand::Register(_) => (),
                    _ => panic!("Memory operands not supported yet!"),
                }
                let group_op = (modrm & 0x38) >> 3;
                match group_op {
                    0 => {
                        println!("inc rm8");
                        if let Operand::Register(opcode_reg) = opcode_params.rm {
                            let reg: u8 = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                            let result = reg.wrapping_add(1);
                            self.regs.flags.set(Flags::ZERO, result == 0);
                            self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                            self.set_parity_flag(result as u16);
                            self.regs.flags.set(Flags::OVERFLOW, ((result ^ 1) & (result ^ reg) & 0x80) == 0x80);
                            self.regs.flags.set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                            self.regs.write8(Reg8::from_num(opcode_reg).unwrap(), result);
                        }
                    }
                    1 => {
                        println!("dec rm8");
                        if let Operand::Register(opcode_reg) = opcode_params.rm {
                            let reg: u8 = self.regs.read8(Reg8::from_num(opcode_reg).unwrap());
                            let result = reg.wrapping_sub(1);
                            self.regs.flags.set(Flags::ZERO, result == 0);
                            self.regs.flags.set(Flags::SIGN, (result & 0x80) == 0x80);
                            self.set_parity_flag(result as u16);
                            self.regs.flags.set(Flags::OVERFLOW, ((reg ^ 1) & (result ^ reg) & 0x80) == 0x80);
                            self.regs.flags.set(Flags::ADJUST, ((result ^ reg ^ 1) & 0x10) == 0x10);
                            self.regs.write8(Reg8::from_num(opcode_reg).unwrap(), result);
                        }
                    }
                    _ => panic!("Unimplemented group opcode!"),
                }
            }
            _ => panic!("Unhandled opcode!"),
        }
        self.seg_override = None;
    }
}
