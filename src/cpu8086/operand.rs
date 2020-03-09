use crate::cpu8086::registers::*;
use crate::cpu8086::Cpu8086;

#[derive(PartialEq, Debug)]
pub enum AddrType {
    BxSi,
    BxDi,
    BpSi,
    BpDi,
    Si,
    Di,
    Bp,
    Bx,
}

#[derive(PartialEq, Debug)]
pub enum DisplacementType {
    Byte,
    Word,
}

#[derive(PartialEq, Debug)]
pub enum Operand {
    Register(u8),
    Address(AddrType, DisplacementType, u16, u16),
    DirectAddr(u16),
    ImmWord(u16),
    ImmByte(u8),
}

pub type ByteOperand = Operand;
pub type WordOperand = Operand;

#[derive(PartialEq, Debug)]
pub struct OpcodeParams {
    pub reg: u8,
    pub rm: Operand,
}

impl Cpu8086 {
    pub fn get_offset(&self, addr_type: AddrType, offset: u16) -> u16 {
        let base = match addr_type {
            AddrType::BxSi => self.regs.read16(Reg16::BX) + self.regs.read16(Reg16::SI),
            AddrType::BxDi => self.regs.read16(Reg16::BX) + self.regs.read16(Reg16::DI),
            AddrType::BpSi => self.regs.read16(Reg16::BP) + self.regs.read16(Reg16::SI),
            AddrType::BpDi => self.regs.read16(Reg16::BP) + self.regs.read16(Reg16::DI),
            AddrType::Si => self.regs.read16(Reg16::SI),
            AddrType::Di => self.regs.read16(Reg16::DI),
            AddrType::Bp => self.regs.read16(Reg16::BP),
            AddrType::Bx => self.regs.read16(Reg16::BX),
        };
        base + offset
    }
    pub fn get_operand_seg(&self, addr_type: AddrType, disp_type: Option<DisplacementType>) -> u16 {
        match self.seg_override {
            Some(segment) => self.regs.readseg16(segment),
            None => match addr_type {
                AddrType::BpSi => self.regs.readseg16(SegReg::SS),
                AddrType::BpDi => self.regs.readseg16(SegReg::SS),
                AddrType::Bp => {
                    if disp_type == Some(DisplacementType::Word) {
                        self.regs.readseg16(SegReg::SS)
                    } else {
                        self.regs.readseg16(SegReg::DS)
                    }
                }
                _ => self.regs.readseg16(SegReg::DS),
            },
        }
    }

    pub fn get_opcode_params_from_modrm(&self, modrm: u8) -> OpcodeParams {
        let mode = (modrm & 0xc0) >> 6;
        let reg = (modrm & 0x38) >> 3;
        let rm = modrm & 7;

        match mode {
            3 => {
                let operand_rm = Operand::Register(rm);
                OpcodeParams {
                    reg: reg,
                    rm: operand_rm,
                }
            }
            _ => panic!("Unimplemented ModR/M mode!"),
        }
    }
}
