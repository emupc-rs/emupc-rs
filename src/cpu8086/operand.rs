use super::cpu8086::Cpu8086;
use registers::*;

#[derive(PartialEq, Clone, Copy, Debug)]
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

pub enum DisplacementType {
    Byte,
    Word,
}

#[derive(PartialEq, Debug)]
pub enum Operand<T> {
    Register(T),
    Address(AddrType, u16)
    DirectAddr(u16),
    ImmWord(u16),
    ImmByte(u8),
}

pub type ByteOperand = Operand<Reg8>;
pub type WordOperand = Operand<Reg16>;

impl Cpu8086 {
    pub fn get_offset(&self, addr_type: AddrType, offset: u16) -> u16 {
        let base = match addr_type {
            AddrType::BxSi => self.registers.read16(Reg16::BX) + self.registers.read16(Reg16::SI),
            AddrType::BxDi => self.registers.read16(Reg16::BX) + self.registers.read16(Reg16::DI),
            AddrType::BpSi => self.registers.read16(Reg16::BP) + self.registers.read16(Reg16::SI),
            AddrType::BpDi => self.registers.read16(Reg16::BP) + self.registers.read16(Reg16::DI),
            AddrType::Si => self.registers.read16(Reg16::SI),
            AddrType::Di => self.registers.read16(Reg16::DI),
            AddrType::Bp => self.registers.read16(Reg16::BP),
            AddrType::Bx => self.registers.read16(Reg16::BX),
        }
        base + offset
    }
    pub fn get_operand_seg(&self, addr_type: AddrType, disp_type: Option<DisplacementType>,
        seg_override: Option<SegReg>) -> u16 {
        match seg_override {
            Some(segment) => self.registers.readseg16(segment),
            None => {
                match AddrType {
                    AddrType::BpSi => self.registers.readseg16(SS),
                    AddrType::BpDi => self.registers.readseg16(SS),
                    AddrType::Bp => {
                        if disp_type == Some(DisplacementType::Word) {
                            self.registers.readseg16(SS)
                        }
                        else {
                            self.registers.readseg16(DS)
                        }
                    },
                    _ => self.registers.readseg16(DS)
                }
            },
        }
    }
}