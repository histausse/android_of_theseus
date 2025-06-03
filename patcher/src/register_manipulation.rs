use androscalpel::{Instruction, RegType};
use anyhow::{bail, Result};
use log::debug;

/// Information about the register used.
///
/// `array_index` and `array` are simple 4 bits register (that is, registers between 0 and 15
/// included that store 32 bit scalar or object depending on the situation) and `pub array_val` is
/// a wide 4 bit register (that is, a register between 0 and 15 included plus the next register, so
/// that it can store 64 bits sclarars in addition to 32 bits scalars and objects depending on the
/// situation). In theory, those should be encoded in u4 types, but rust does not have those.
///
/// Because we can rarely reserved 4 bits registers for a whole method, `array_index_save`, `array_val_save`
/// and `array_save` are 16 bits registers where we can save the previous contant of the registers
/// before using them.
///
/// `first_arg` is the first register of plage of `nb_arg_reg` use to invoke method.
#[derive(PartialEq, Debug, Default)]
pub(crate) struct RegistersInfo {
    pub array_index: u8,
    pub array: u8,
    pub array_val: u8, // Reserver 2 reg here, for wide operation
    pub array_index_save: Option<u16>,
    pub array_save: Option<u16>,
    pub array_val_save: Option<u16>, // Reserver 2 reg here, for wide operation
    pub first_arg: u16,
    pub nb_arg_reg: u16,
}

impl RegistersInfo {
    pub fn get_nb_added_reg(&self) -> u16 {
        self.nb_arg_reg + 4
    }

    /// Set the values for `array_index`, `array` and `array_val` when the methode already use more
    /// than 12 registers. This means already used registers need to be saved in order to be used.
    /// The first instruction vec return contains the instructions to save the registers, the
    /// second the instructions to restore the registers to their old values.
    ///
    /// `used_reg` is a list of register that cannot be used because directly used by the invoke
    /// instruction or the move-result instruction.
    /// `regs_type` is the type of the registers at this point in the code of the method.
    pub fn tmp_reserve_reg(
        &mut self,
        used_reg: &[u16],
        regs_type: &[RegType],
    ) -> Result<(Vec<Instruction>, Vec<Instruction>)> {
        let mut used_reg = used_reg.to_vec();
        let mut save_reg_insns = vec![];
        let mut restore_reg_insns = vec![];
        if let Some(reg_save) = self.array_val_save {
            let mut found = false;
            if reg_save <= 0b1110 {
                // This should not happend, but who knows?
                found = true;
            }
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16))
                        && !used_reg.contains(&((i + 1) as u16))
                        && regs_type[i] == RegType::FirstWideScalar
                        && regs_type[i + 1] == RegType::SecondWideScalar
                    {
                        self.array_val = i as u8;
                        used_reg.push(i as u16);
                        used_reg.push((i + 1) as u16);
                        save_reg_insns.push(Instruction::MoveWide {
                            from: i as u16,
                            to: reg_save,
                        });
                        restore_reg_insns.push(Instruction::MoveWide {
                            from: reg_save,
                            to: i as u16,
                        });
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16))
                        && !used_reg.contains(&((i + 1) as u16))
                        && (regs_type[i] == RegType::Object
                            || regs_type[i] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::Undefined)
                        && (regs_type[i + 1] == RegType::Object
                            || regs_type[i + 1] == RegType::SimpleScalar
                            || regs_type[i + 1] == RegType::FirstWideScalar
                            || regs_type[i + 1] == RegType::SecondWideScalar
                            || regs_type[i + 1] == RegType::Undefined)
                    {
                        self.array_val = i as u8;
                        used_reg.push(i as u16);
                        used_reg.push((i + 1) as u16);
                        if regs_type[i] == RegType::Object {
                            save_reg_insns.push(Instruction::MoveObject {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::MoveObject {
                                from: reg_save,
                                to: i as u16,
                            });
                        } else if regs_type[i] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                        {
                            save_reg_insns.push(Instruction::Move {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::Move {
                                from: reg_save,
                                to: i as u16,
                            });
                        } // else RegType::Undefined, do nothing, just use it
                        if regs_type[i + 1] == RegType::Object {
                            save_reg_insns.push(Instruction::MoveObject {
                                from: (i + 1) as u16,
                                to: reg_save + 1,
                            });
                            restore_reg_insns.push(Instruction::MoveObject {
                                from: reg_save + 1,
                                to: (i + 1) as u16,
                            });
                        } else if regs_type[i + 1] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                        {
                            save_reg_insns.push(Instruction::Move {
                                from: (i + 1) as u16,
                                to: reg_save + 1,
                            });
                            restore_reg_insns.push(Instruction::Move {
                                from: reg_save + 1,
                                to: (i + 1) as u16,
                            });
                        } // else RegType::Undefined, do nothing, just use it
                        found = true;
                        break;
                    }
                }
            }
            // Last resort
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16))
                        && !used_reg.contains(&((i + 1) as u16))
                        && (regs_type[i] == RegType::Object
                            || regs_type[i] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::Any
                            || regs_type[i] == RegType::Undefined)
                        && (regs_type[i + 1] == RegType::Object
                            || regs_type[i + 1] == RegType::SimpleScalar
                            || regs_type[i + 1] == RegType::FirstWideScalar
                            || regs_type[i + 1] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::Any
                            || regs_type[i + 1] == RegType::Undefined)
                    {
                        self.array_val = i as u8;
                        used_reg.push(i as u16);
                        used_reg.push((i + 1) as u16);
                        if regs_type[i] == RegType::Object {
                            save_reg_insns.push(Instruction::MoveObject {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::MoveObject {
                                from: reg_save,
                                to: i as u16,
                            });
                        } else if regs_type[i] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::Any
                        {
                            save_reg_insns.push(Instruction::Move {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::Move {
                                from: reg_save,
                                to: i as u16,
                            });
                        } // else RegType::Undefined, do nothing, just use it
                        if regs_type[i + 1] == RegType::Object {
                            save_reg_insns.push(Instruction::MoveObject {
                                from: (i + 1) as u16,
                                to: reg_save + 1,
                            });
                            restore_reg_insns.push(Instruction::MoveObject {
                                from: reg_save + 1,
                                to: (i + 1) as u16,
                            });
                        } else if regs_type[i + 1] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                        {
                            save_reg_insns.push(Instruction::Move {
                                from: (i + 1) as u16,
                                to: reg_save + 1,
                            });
                            restore_reg_insns.push(Instruction::Move {
                                from: reg_save + 1,
                                to: (i + 1) as u16,
                            });
                        } // else RegType::Undefined, do nothing, just use it
                        found = true;
                        break;
                    }
                }
                if !found {
                    bail!("Could not found enough usable registers to patch the method")
                }
            }
            debug!(
                "Temporarily reserve registers {}-{} and save their values to {}-{}",
                self.array_val,
                self.array_val + 1,
                reg_save,
                reg_save + 1
            );
        }
        if let Some(reg_save) = self.array_index_save {
            let mut found = false;
            if reg_save <= 0b1111 {
                // This should not happend, but who knows?
                found = true;
            }
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16)) && regs_type[i] == RegType::SimpleScalar {
                        self.array_index = i as u8;
                        used_reg.push(i as u16);
                        save_reg_insns.push(Instruction::Move {
                            from: i as u16,
                            to: reg_save,
                        });
                        restore_reg_insns.push(Instruction::Move {
                            from: reg_save,
                            to: i as u16,
                        });
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16))
                        && (regs_type[i] == RegType::Object
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::Undefined)
                    {
                        self.array_index = i as u8;
                        used_reg.push(i as u16);
                        if regs_type[i] == RegType::Object {
                            save_reg_insns.push(Instruction::MoveObject {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::MoveObject {
                                from: reg_save,
                                to: i as u16,
                            });
                        } else if regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                        {
                            save_reg_insns.push(Instruction::Move {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::Move {
                                from: reg_save,
                                to: i as u16,
                            });
                        } // else RegType::Undefined, do nothing, just use it
                        found = true;
                        break;
                    }
                }
            }
            // Last resort
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16)) && regs_type[i] == RegType::Any {
                        self.array_index = i as u8;
                        used_reg.push(i as u16);
                        save_reg_insns.push(Instruction::Move {
                            from: i as u16,
                            to: reg_save,
                        });
                        restore_reg_insns.push(Instruction::Move {
                            from: reg_save,
                            to: i as u16,
                        });
                        found = true;
                        break;
                    }
                }
                if !found {
                    bail!("Could not found enough usable registers to patch the method")
                }
            }
            debug!(
                "Temporarily reserve register {} and save it value to {}",
                self.array_index, reg_save,
            );
        }
        if let Some(reg_save) = self.array_save {
            let mut found = false;
            if reg_save <= 0b1111 {
                // This should not happend, but who knows?
                found = true;
            }
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16)) && regs_type[i] == RegType::Object {
                        self.array = i as u8;
                        used_reg.push(i as u16);
                        save_reg_insns.push(Instruction::MoveObject {
                            from: i as u16,
                            to: reg_save,
                        });
                        restore_reg_insns.push(Instruction::MoveObject {
                            from: reg_save,
                            to: i as u16,
                        });
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16))
                        && (regs_type[i] == RegType::SimpleScalar
                            || regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::Undefined)
                    {
                        self.array = i as u8;
                        used_reg.push(i as u16);
                        if regs_type[i] == RegType::FirstWideScalar
                            || regs_type[i] == RegType::SecondWideScalar
                            || regs_type[i] == RegType::SimpleScalar
                        {
                            save_reg_insns.push(Instruction::Move {
                                from: i as u16,
                                to: reg_save,
                            });
                            restore_reg_insns.push(Instruction::Move {
                                from: reg_save,
                                to: i as u16,
                            });
                        } // else RegType::Undefined, do nothing, just use it
                        found = true;
                        break;
                    }
                }
            }
            // Last resort
            if !found {
                for i in 0..15 {
                    if i >= regs_type.len() {
                        break;
                    }
                    if !used_reg.contains(&(i as u16)) && regs_type[i] == RegType::Any {
                        self.array = i as u8;
                        used_reg.push(i as u16);
                        save_reg_insns.push(Instruction::Move {
                            from: i as u16,
                            to: reg_save,
                        });
                        restore_reg_insns.push(Instruction::Move {
                            from: reg_save,
                            to: i as u16,
                        });
                        found = true;
                        break;
                    }
                }
                if !found {
                    bail!("Could not found enough usable registers to patch the method")
                }
            }
            debug!(
                "Temporarily reserve register {} and save it value to {}",
                self.array, reg_save,
            );
        }
        Ok((save_reg_insns, restore_reg_insns))
    }
}
