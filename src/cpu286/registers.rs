use bitflags::bitflags;
use std::fmt;
#[allow(dead_code)]

bitflags!(
    pub struct Flags: u16
    {
        const CARRY = 0x0001;
        const PARITY = 0x0004;
        const ADJUST = 0x0010;
        const ZERO = 0x0040;
        const SIGN = 0x0080;
        const TRAP = 0x0100;
        const INTERRUPT = 0x0200;
        const DIRECTION = 0x0400;
        const OVERFLOW = 0x0800;
        const IOPL = 0x3000;
        const NESTED_TASK = 0x4000;
        const DEFAULT = 0x0002;
    }
);

impl Default for Flags {
    fn default() -> Flags {
        Flags::DEFAULT
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Reg8 {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
}

#[derive(Clone, Copy, Debug)]
pub enum Reg16 {
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
    FLAGS,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SegReg {
    ES,
    CS,
    SS,
    DS,
}

#[derive(Clone, Copy, Debug)]
pub enum TableReg {
    GDTR,
    IDTR,
    LDTR,
    TR,
}

#[derive(Clone, Copy, Debug)]
pub struct SegmentRegister {
    pub selector: u16,
    pub base: u32, //Actually only 24 bits
    pub limit: u16,
    pub rights: u8,
    pub valid: bool,
}

impl SegmentRegister {
    pub fn new(seg: SegReg) -> SegmentRegister {
        SegmentRegister {
            selector: if seg == SegReg::CS { 0xf000 } else { 0 },
            base: if seg == SegReg::CS { 0xff0000 } else { 0 },
            limit: 0xffff,
            rights: 0x93,
            valid: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GDTRIDTR {
    pub base: u32, //Actually only 24 bits
    pub limit: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct LDTRTR {
    pub selector: u16,
    pub base: u32, //Actually only 24 bits
    pub limit: u16,
    pub rights: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Registers {
    pub ip: u16,
    pub gprs: [u16; 8],
    pub seg_regs: [SegmentRegister; 4],
    pub flags: Flags,
    pub msw: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            ip: 0xfff0,
            gprs: [0; 8],
            seg_regs: [
                SegmentRegister::new(SegReg::ES),
                SegmentRegister::new(SegReg::CS),
                SegmentRegister::new(SegReg::SS),
                SegmentRegister::new(SegReg::DS),
            ],
            flags: Flags::empty(),
            msw: 0xfff0,
        }
    }

    pub fn read8(&self, reg: Reg8) -> u8 {
        use self::Reg8::*;
        match reg {
            AL => (self.gprs[0] & 0xff) as u8,
            CL => (self.gprs[1] & 0xff) as u8,
            DL => (self.gprs[2] & 0xff) as u8,
            BL => (self.gprs[3] & 0xff) as u8,
            AH => (self.gprs[0] >> 8) as u8,
            CH => (self.gprs[1] >> 8) as u8,
            DH => (self.gprs[2] >> 8) as u8,
            BH => (self.gprs[3] >> 8) as u8,
        }
    }

    pub fn write8(&mut self, reg: Reg8, value: u8) {
        use self::Reg8::*;
        match reg {
            AL => {
                self.gprs[0] &= !0xffu16;
                self.gprs[0] |= value as u16;
            }
            CL => {
                self.gprs[1] &= !0xffu16;
                self.gprs[1] |= value as u16;
            }
            DL => {
                self.gprs[2] &= !0xffu16;
                self.gprs[2] |= value as u16;
            }
            BL => {
                self.gprs[3] &= !0xffu16;
                self.gprs[3] |= value as u16;
            }
            AH => {
                self.gprs[0] &= 0xffu16;
                self.gprs[0] |= (value as u16) << 8;
            }
            CH => {
                self.gprs[1] &= 0xffu16;
                self.gprs[1] |= (value as u16) << 8;
            }
            DH => {
                self.gprs[2] &= 0xffu16;
                self.gprs[2] |= (value as u16) << 8;
            }
            BH => {
                self.gprs[3] &= 0xffu16;
                self.gprs[3] |= (value as u16) << 8;
            }
        }
    }

    pub fn read16(&self, reg: Reg16) -> u16 {
        use self::Reg16::*;
        match reg {
            AX => self.gprs[0],
            CX => self.gprs[1],
            DX => self.gprs[2],
            BX => self.gprs[3],
            SP => self.gprs[4],
            BP => self.gprs[5],
            SI => self.gprs[6],
            DI => self.gprs[7],
            FLAGS => (self.flags.bits() as u16) | 0xf002u16,
        }
    }

    pub fn write16(&mut self, reg: Reg16, value: u16) {
        use self::Reg16::*;
        match reg {
            AX => self.gprs[0] = value,
            CX => self.gprs[1] = value,
            DX => self.gprs[2] = value,
            BX => self.gprs[3] = value,
            SP => self.gprs[4] = value,
            BP => self.gprs[5] = value,
            SI => self.gprs[6] = value,
            DI => self.gprs[7] = value,
            FLAGS => self.flags = Flags::from_bits_truncate(value),
        }
    }

    pub fn readseg16(&self, seg_reg: SegReg) -> SegmentRegister {
        use self::SegReg::*;
        match seg_reg {
            ES => self.seg_regs[0],
            CS => self.seg_regs[1],
            SS => self.seg_regs[2],
            DS => self.seg_regs[3],
        }
    }

    pub fn writeseg16(&mut self, seg_reg: SegReg, value: u16) {
        use self::SegReg::*;
        if (self.msw & 1) != 0 {
            panic!("Protected mode not implemented yet!");
        } else {
            let mut segment = 1;
            match seg_reg {
                ES => segment = 0,
                CS => segment = 1,
                SS => segment = 2,
                DS => segment = 3,
            }
            self.seg_regs[segment].selector = value;
            self.seg_regs[segment].base = (value as u32) << 4;
        }
    }
}
