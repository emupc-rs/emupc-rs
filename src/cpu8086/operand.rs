use crate::cpu8086::registers::*;
use crate::cpu8086::Cpu8086;
use crate::cpu8086::Cpu8086Context;

#[derive(PartialEq, Debug, Clone, Copy)]
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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DisplacementType {
    Byte,
    Word,
}

#[derive(PartialEq, Debug)]
pub enum Operand {
    Register(u8),
    Address(SegReg, u16),
}

pub type ByteOperand = Operand;
pub type WordOperand = Operand;

#[derive(PartialEq, Debug)]
pub struct OpcodeParams {
    pub reg: u8,
    pub rm: Operand,
}

impl Cpu8086 {
    pub fn get_addr_type_from_modrm(modrm: u8) -> Option<AddrType> {
        let mode = (modrm & 0xc0) >> 6;
        let rm = modrm & 7;
        match rm {
            0 => Some(AddrType::BxSi),
            1 => Some(AddrType::BxDi),
            2 => Some(AddrType::BpSi),
            3 => Some(AddrType::BpDi),
            4 => Some(AddrType::Si),
            5 => Some(AddrType::Di),
            6 => {
                if mode == 0 {
                    None
                } else {
                    Some(AddrType::Bp)
                }
            }
            7 => Some(AddrType::Bx),
            _ => panic!("Invalid address type!"),
        }
    }
    pub fn get_disp_type_from_modrm(modrm: u8) -> Option<DisplacementType> {
        let mode = (modrm & 0xc0) >> 6;
        let rm = modrm & 7;
        match mode {
            0 => {
                if rm == 6 {
                    Some(DisplacementType::Word)
                } else {
                    None
                }
            }
            1 => Some(DisplacementType::Byte),
            2 => Some(DisplacementType::Word),
            _ => panic!("Invalid displacement type!"),
        }
    }
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
    pub fn get_operand_seg(
        &self,
        addr_type: Option<AddrType>,
        disp_type: Option<DisplacementType>,
    ) -> SegReg {
        match self.seg_override {
            Some(segment) => segment,
            None => match addr_type {
                Some(AddrType::BpSi) => SegReg::SS,
                Some(AddrType::BpDi) => SegReg::SS,
                Some(AddrType::Bp) => {
                    if disp_type == Some(DisplacementType::Word) {
                        SegReg::SS
                    } else {
                        SegReg::DS
                    }
                }
                _ => SegReg::DS,
            },
        }
    }

    pub fn get_opcode_params_from_modrm<T: Cpu8086Context>(
        &mut self,
        ctx: &mut T,
        modrm: u8,
    ) -> OpcodeParams {
        let mode = (modrm & 0xc0) >> 6;
        let reg = (modrm & 0x38) >> 3;
        let rm = modrm & 7;

        match mode {
            0 => {
                let addr_type = Cpu8086::get_addr_type_from_modrm(modrm);
                let disp_type = Cpu8086::get_disp_type_from_modrm(modrm);
                let addr: u16;
                let displacement: u16;
                let segment: SegReg = self.get_operand_seg(addr_type, disp_type);
                match disp_type {
                    None => {
                        displacement = 0;
                    }
                    Some(DisplacementType::Byte) => {
                        displacement =
                            self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip)
                                as u16;
                        self.regs.ip = self.regs.ip.wrapping_add(1);
                    }
                    Some(DisplacementType::Word) => {
                        displacement =
                            self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip);
                        self.regs.ip = self.regs.ip.wrapping_add(2);
                    }
                }
                match addr_type {
                    None => {
                        addr = displacement;
                    }
                    Some(_) => {
                        addr = self.get_offset(addr_type.unwrap(), displacement);
                    }
                }
                let operand_rm = Operand::Address(segment, addr);
                let operand_reg = reg;
                OpcodeParams {
                    reg: operand_reg,
                    rm: operand_rm,
                }
            }
            1 => {
                let addr_type = Cpu8086::get_addr_type_from_modrm(modrm);
                let addr: u16;
                let displacement: u16 =
                    self.mem_read_byte(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip) as u16;
                let segment: SegReg = self.get_operand_seg(addr_type, Some(DisplacementType::Byte));
                self.regs.ip = self.regs.ip.wrapping_add(1);
                match addr_type {
                    None => panic!("Invalid address type for this ModR/M type!"),
                    Some(_) => {
                        addr = self.get_offset(addr_type.unwrap(), displacement);
                    }
                }
                let operand_rm = Operand::Address(segment, addr);
                let operand_reg = reg;
                OpcodeParams {
                    reg: operand_reg,
                    rm: operand_rm,
                }
            }
            2 => {
                let addr_type = Cpu8086::get_addr_type_from_modrm(modrm);
                let addr: u16;
                let displacement: u16 =
                    self.mem_read_word(ctx, self.regs.readseg16(SegReg::CS), self.regs.ip);
                let segment: SegReg = self.get_operand_seg(addr_type, Some(DisplacementType::Word));
                self.regs.ip = self.regs.ip.wrapping_add(2);
                match addr_type {
                    None => panic!("Invalid address type for this ModR/M type!"),
                    Some(_) => {
                        addr = self.get_offset(addr_type.unwrap(), displacement);
                    }
                }
                let operand_rm = Operand::Address(segment, addr);
                let operand_reg = reg;
                OpcodeParams {
                    reg: operand_reg,
                    rm: operand_rm,
                }
            }
            3 => {
                let operand_rm = Operand::Register(rm);
                let operand_reg = reg;
                OpcodeParams {
                    reg: operand_reg,
                    rm: operand_rm,
                }
            }
            _ => panic!("Unimplemented ModR/M mode!"),
        }
    }
}

#[test]
fn test_modrm() {
    let mut machine = IbmPc5150Machine::new();
    for modrm in 0..=0xffu8 {
        machine
            .cpu
            .get_opcode_params_from_modrm(&mut machine.hardware, modrm);
    }
}
