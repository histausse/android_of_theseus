use androscalpel::SmaliName;
use androscalpel::{Code, IdMethod, IdMethodType, IdType, Instruction, Method};
use anyhow::{bail, Context, Result};
use log::warn;

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{dex_types::*, register_manipulation::*, runtime_data::*};

// Interesting stuff: https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/verifier/reg_type.h;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=94
// https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/verifier/method_verifier.cc;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=5328
/// `meth`: the method that make reflectif calls. This is the method to patch.
/// `ref_data`: the runtime data containing the reflectif calls informations.
/// `tester_methods_class`: the class used to define the methods in `tester_methods`
/// `tester_methods`: the methods used to test if a `java.lang.reflect.Method` or `java.lang.reflect.Constructor`
///     is a specific method. Methods are indexed by the IdMethod they detect, and have a name derived from the method
///     they detect.
pub fn transform_method(
    meth: &mut Method,
    ref_data: &RuntimeData,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<IdMethod, Method>,
) -> Result<()> {
    // checking meth.annotations might be usefull at some point
    //println!("{}", meth.descriptor.__str__());
    let invoke_data = ref_data.get_invoke_data_for(&meth.descriptor);
    let class_new_inst_data = ref_data.get_class_new_instance_data_for(&meth.descriptor);
    let cnstr_new_inst_data = ref_data.get_cnstr_new_instance_data_for(&meth.descriptor);

    let code = meth
        .code
        .as_ref()
        .with_context(|| format!("Code not found in {}", meth.descriptor.__str__()))?;

    // Get the available registers at the method level
    let mut register_info = RegistersInfo::default();
    // register_info.array_val is a wide reg, so need at least 0b1110 and 0b1111
    if code.registers_size < 0b1111 {
        register_info.array_val = code.registers_size as u8;
    } else {
        register_info.array_val = 0;
        register_info.array_val_save = Some(code.registers_size);
    }
    if code.registers_size + 2 <= 0b1111 {
        register_info.array_index = (code.registers_size + 2) as u8;
    } else {
        register_info.array_index = 0;
        register_info.array_index_save = Some(code.registers_size + 2);
    }
    if code.registers_size + 3 <= 0b1111 {
        register_info.array = (code.registers_size + 3) as u8;
    } else {
        register_info.array = 0;
        register_info.array_save = Some(code.registers_size + 3);
    }
    register_info.first_arg = code.registers_size + 4;
    register_info.nb_arg_reg = 0;

    let regs_type = if register_info.array_val_save.is_some()
        || register_info.array_index_save.is_some()
        || register_info.array_save.is_some()
    {
        Some(meth.get_cfg()?.get_reg_types())
    } else {
        None
    };

    let mut new_insns = vec![];
    let mut iter = code.insns.iter();
    let mut current_addr_label: Option<String> = None;
    while let Some(ins) = iter.next() {
        match ins {
            Instruction::InvokeVirtual { method, args }
                if (method == &*MTH_INVOKE
                    || method == &*CLASS_NEW_INST
                    || method == &*CNSTR_NEW_INST)
                    && current_addr_label.is_some() =>
            {
                let addr_label = current_addr_label.as_ref().unwrap();
                let (pseudo_insns, move_ret) = get_move_result(iter.clone());
                if move_ret.is_some() {
                    while move_ret.as_ref() != iter.next() {}
                }
                let end_label = if method == &*MTH_INVOKE {
                    format!("end_reflection_call_at_{}", "TODO_ADDR")
                } else if method == &*CLASS_NEW_INST || method == &*CNSTR_NEW_INST {
                    format!("end_reflection_instanciation_at_{}", "TODO_ADDR")
                } else {
                    panic!("Should not happen!")
                };
                let mut restore_reg = vec![];
                if let Some(regs_type) = regs_type.as_ref() {
                    if (method == &*MTH_INVOKE && invoke_data.contains_key(addr_label))
                        || (method == &*CLASS_NEW_INST
                            && class_new_inst_data.contains_key(addr_label))
                        || (method == &*CNSTR_NEW_INST
                            && cnstr_new_inst_data.contains_key(addr_label))
                    {
                        let regs_type = regs_type.get(addr_label).unwrap();
                        let mut used_reg = args.clone();
                        match move_ret {
                            Some(Instruction::MoveResult { to }) => used_reg.push(to as u16),
                            Some(Instruction::MoveResultObject { to }) => used_reg.push(to as u16),
                            Some(Instruction::MoveResultWide { to }) => used_reg.push(to as u16),
                            _ => (),
                        }
                        match register_info.tmp_reserve_reg(&used_reg, regs_type) {
                            Ok((mut save_insns, restore_insns)) => {
                                restore_reg = restore_insns;
                                new_insns.append(&mut save_insns);
                            }
                            Err(err) => {
                                warn!(
                                    "Failed to instrument reflection in {} at {}: {}",
                                    method.__str__(),
                                    addr_label,
                                    err,
                                );
                                new_insns.push(ins.clone());
                                if let Some(move_ret) = move_ret.as_ref() {
                                    for ins in pseudo_insns.iter() {
                                        new_insns.push(ins.clone());
                                    }
                                    new_insns.push(move_ret.clone());
                                }
                                current_addr_label = None;
                                continue;
                            }
                        }
                    }
                }
                // TODO: recover from failure
                if method == &*MTH_INVOKE {
                    for ref_data in invoke_data.get(addr_label).unwrap_or(&vec![]) {
                        for ins in get_invoke_block(
                            ref_data,
                            args.as_slice(),
                            &mut register_info,
                            &end_label,
                            move_ret.clone(),
                            tester_methods_class.clone(),
                            tester_methods,
                        )? {
                            new_insns.push(ins);
                        }
                    }
                } else if method == &*CLASS_NEW_INST {
                    for ref_data in class_new_inst_data.get(addr_label).unwrap_or(&vec![]) {
                        for ins in get_class_new_inst_block(
                            ref_data,
                            args.as_slice(),
                            &mut register_info,
                            &end_label,
                            move_ret.clone(),
                        )? {
                            new_insns.push(ins);
                        }
                    }
                } else if method == &*CNSTR_NEW_INST {
                    for ref_data in cnstr_new_inst_data.get(addr_label).unwrap_or(&vec![]) {
                        for ins in get_cnstr_new_inst_block(
                            ref_data,
                            args.as_slice(),
                            &mut register_info,
                            &end_label,
                            move_ret.clone(),
                            tester_methods_class.clone(),
                            tester_methods,
                        )? {
                            new_insns.push(ins);
                        }
                    }
                } else {
                    panic!("Should not happen!")
                };
                new_insns.push(ins.clone());
                if let Some(move_ret) = move_ret {
                    for ins in pseudo_insns.into_iter() {
                        new_insns.push(ins);
                    }
                    new_insns.push(move_ret);
                }
                let end_label = Instruction::Label { name: end_label };
                new_insns.push(end_label.clone());
                new_insns.append(&mut restore_reg);
                current_addr_label = None;
            }
            Instruction::Label { name } if name.starts_with("THESEUS_ADDR_") => {
                current_addr_label = Some(name.clone());
                new_insns.push(ins.clone());
            }
            ins => {
                if !ins.is_pseudo_ins() {
                    current_addr_label = None;
                }
                new_insns.push(ins.clone());
            }
        }
    }
    let ins_size = code.ins_size(meth);
    let code = meth
        .code
        .as_mut()
        .with_context(|| format!("Code not found in {}", meth.descriptor.__str__()))?;

    code.insns = vec![];
    // Start the method by moving the parameter to their registers pre-transformation.
    let mut i = 0;
    if !meth.is_static {
        // Non static method take 'this' as first argument
        code.insns.push(Instruction::MoveObject {
            from: code.registers_size - ins_size + i + register_info.get_nb_added_reg(),
            to: code.registers_size - ins_size + i,
        });
        i += 1;
    }
    for arg in &meth.descriptor.proto.get_parameters() {
        if arg.is_class() || arg.is_array() {
            code.insns.push(Instruction::MoveObject {
                from: code.registers_size - ins_size + i + register_info.get_nb_added_reg(),
                to: code.registers_size - ins_size + i,
            });
            i += 1;
        } else if arg.is_long() || arg.is_double() {
            code.insns.push(Instruction::MoveWide {
                from: code.registers_size - ins_size + i + register_info.get_nb_added_reg(),
                to: code.registers_size - ins_size + i,
            });
            i += 2;
        } else {
            code.insns.push(Instruction::Move {
                from: code.registers_size - ins_size + i + register_info.get_nb_added_reg(),
                to: code.registers_size - ins_size + i,
            });
            i += 1;
        }
    }
    if i != ins_size {
        warn!(
            "Method {} argument do not match code ins_size ({})",
            meth.descriptor.__str__(),
            ins_size
        );
    }
    // Add the new code
    code.insns.append(&mut new_insns);
    code.registers_size += register_info.get_nb_added_reg();

    Ok(())
}

fn gen_tester_method(
    tester_methods_class: IdType,
    method_to_test: IdMethod,
    is_constructor: bool,
) -> Result<Method> {
    let mut hasher = DefaultHasher::new();
    method_to_test.hash(&mut hasher);
    let hash = hasher.finish();
    let m_name: String = (&method_to_test.name).try_into()?;
    let m_name = m_name.replace("<", "").replace(">", "");
    let c_name = {
        let class: String = match method_to_test.class_.get_class_name() {
            None => method_to_test.class_.try_to_smali()?,
            Some(class) => class.try_into()?,
        };
        match class.rsplit_once('/') {
            None => class,
            Some((_, name)) => name.to_string(),
        }
    };

    let descriptor = IdMethod::new(
        format!("check_is_{c_name}_{m_name}_{hash:016x}").into(),
        IdMethodType::new(
            IdType::boolean(),
            vec![if is_constructor {
                IdType::class("java/lang/reflect/Constructor")
            } else {
                IdType::class("java/lang/reflect/Method")
            }],
        ),
        tester_methods_class,
    );
    let mut method = Method::new(descriptor);
    let no_label: String = "lable_no".into();
    let reg_arr = 0;
    let reg_arr_idx = 1;
    let reg_arr_val = 2;
    let reg_ref_method = 3;
    // Check for arg type
    let mut insns = if !is_constructor {
        vec![
            Instruction::InvokeVirtual {
                method: MTH_GET_PARAMS_TY.clone(),
                args: vec![reg_ref_method],
            },
            Instruction::MoveResultObject { to: reg_arr },
        ]
    } else {
        vec![
            Instruction::InvokeVirtual {
                method: CNSTR_GET_PARAMS_TY.clone(),
                args: vec![reg_ref_method],
            },
            Instruction::MoveResultObject { to: reg_arr },
        ]
    };
    // First check  the number of args
    // --------------------
    insns.append(&mut vec![
        Instruction::ArrayLength {
            dest: reg_arr_idx,
            arr: reg_arr,
        },
        Instruction::Const {
            reg: reg_arr_val,
            lit: method_to_test.proto.get_parameters().len() as i32,
        },
        Instruction::IfNe {
            a: reg_arr_idx,
            b: reg_arr_val,
            label: no_label.clone(),
        },
    ]);
    // then the type of each arg
    for (i, param) in method_to_test
        .proto
        .get_parameters()
        .into_iter()
        .enumerate()
    {
        insns.push(Instruction::Const {
            reg: reg_arr_idx,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: reg_arr_val,
            arr: reg_arr,
            idx: reg_arr_idx,
        });
        insns.push(Instruction::ConstClass {
            reg: reg_arr_idx, // wrong name, but available for tmp val
            lit: param,
        });
        insns.push(Instruction::IfNe {
            a: reg_arr_idx,
            b: reg_arr_val,
            label: no_label.clone(),
        })
    }
    if !is_constructor {
        insns.append(&mut vec![
            // Check the runtime method is the right one
            // Check Name
            Instruction::InvokeVirtual {
                method: MTH_GET_NAME.clone(),
                args: vec![reg_ref_method],
            },
            Instruction::MoveResultObject {
                to: reg_arr_idx, // wrong name, but available for tmp val
            },
            Instruction::ConstString {
                reg: reg_arr_val, // wrong name, but available for tmp val
                lit: method_to_test.name.clone(),
            },
            Instruction::InvokeVirtual {
                method: STR_EQ.clone(),
                args: vec![reg_arr_idx as u16, reg_arr_val as u16],
            },
            Instruction::MoveResult {
                to: reg_arr_idx, // wrong name, but available for tmp val
            },
            Instruction::IfEqZ {
                a: reg_arr_idx,
                label: no_label.clone(),
            },
            // Check Return Type
            Instruction::InvokeVirtual {
                method: MTH_GET_RET_TY.clone(),
                args: vec![reg_ref_method],
            },
            Instruction::MoveResultObject {
                to: reg_arr_idx, // wrong name, but available for tmp val
            },
            Instruction::ConstClass {
                reg: reg_arr_val, // wrong name, but available for tmp val
                lit: method_to_test.proto.get_return_type(),
            },
            Instruction::IfNe {
                a: reg_arr_idx,
                b: reg_arr_val,
                label: no_label.clone(),
            },
            // Check Declaring Type
            Instruction::InvokeVirtual {
                method: MTH_GET_DEC_CLS.clone(),
                args: vec![reg_ref_method],
            },
        ]);
    }
    if is_constructor {
        // Check Declaring Type
        insns.push(Instruction::InvokeVirtual {
            method: CNSTR_GET_DEC_CLS.clone(),
            args: vec![reg_ref_method],
        });
    }
    insns.append(&mut vec![
        Instruction::MoveResultObject {
            to: reg_arr_idx, // wrong name, but available for tmp val
        },
        Instruction::ConstClass {
            reg: reg_arr_val, // wrong name, but available for tmp val
            lit: method_to_test.class_.clone(),
        },
        Instruction::IfNe {
            a: reg_arr_idx,
            b: reg_arr_val,
            label: no_label.clone(),
        },
        Instruction::Const {
            reg: reg_arr_val,
            lit: 1,
        },
        Instruction::Return { reg: reg_arr_val },
        Instruction::Label { name: no_label },
        Instruction::Const {
            reg: reg_arr_val,
            lit: 0,
        },
        Instruction::Return { reg: reg_arr_val },
    ]);

    method.is_static = true;
    method.is_final = true;
    method.code = Some(Code::new(
        4, //registers_size, 3 reg + 1 parameter reg
        insns,
        Some(vec![Some("meth".into())]), // parameter_names
    ));
    Ok(method)
}

/// Generate bytecode that test if a `java.lang.reflect.Method` is equal to an [`IdMethod`]
///
/// - `method_obj_reg`: the register containing the `java.lang.reflect.Method`
/// - `id_method`: the expected [`IdMethod`].
/// - `abort_label`: the label where to jump if the method does not match `id_method`.
/// - `tester_methods_class`: the class used to define the methods in `tester_methods`
/// - `tester_methods`: the methods used to test if a `java.lang.reflect.Method` is a specific method.
///     Methods are indexed by the IdMethod they detect, and have a name derived from the method
///     they detect.
fn test_method(
    method_obj_reg: u16,
    id_method: IdMethod,
    abort_label: String,
    reg_inf: &mut RegistersInfo,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<IdMethod, Method>,
) -> Result<Vec<Instruction>> {
    use std::collections::hash_map::Entry;
    let tst_descriptor = match tester_methods.entry(id_method.clone()) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => e.insert(gen_tester_method(tester_methods_class, id_method, false)?),
    }
    .descriptor
    .clone();
    Ok(vec![
        Instruction::InvokeStatic {
            method: tst_descriptor,
            args: vec![method_obj_reg],
        },
        Instruction::MoveResult {
            to: reg_inf.array_val,
        },
        Instruction::IfEqZ {
            a: reg_inf.array_val,
            label: abort_label,
        },
    ])
}

/// Return the MoveResult{,Wide,Object} associated to the last instruction of the iterator.
fn get_move_result<'a>(
    iter: impl Iterator<Item = &'a Instruction>,
) -> (Vec<Instruction>, Option<Instruction>) {
    let mut pseudo_insns = vec![];
    for ins in iter {
        /*
        match ins {
            Instruction::MoveResult { .. }
            | Instruction::MoveResultWide { .. }
            | Instruction::MoveResultObject { .. } => return (vec![], Some(ins.clone())),
            _ => (), // break,
        }*/
        if ins.is_pseudo_ins() {
            pseudo_insns.push(ins.clone());
        } else if let Instruction::MoveResultObject { .. } = ins {
            return (pseudo_insns, Some(ins.clone()));
        } else {
            break;
        }
    }
    (vec![], None)
}

fn get_invoke_block(
    ref_data: &ReflectionInvokeData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
    end_label: &str,
    move_result: Option<Instruction>,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<IdMethod, Method>,
) -> Result<Vec<Instruction>> {
    let (method_obj, obj_inst, arg_arr) = if let &[a, b, c] = invoke_arg {
        (a, b, c)
    } else {
        bail!(
            "Method;->invoke arg should have exactly 3 arguments, found {}",
            invoke_arg.len()
        );
    };
    let nb_args: usize = ref_data
        .method
        .proto
        .get_parameters()
        .iter()
        .map(|ty| if ty.is_double() || ty.is_long() { 2 } else { 1 })
        .sum();
    if reg_inf.nb_arg_reg < nb_args as u16 + if ref_data.is_static { 0 } else { 1 } {
        reg_inf.nb_arg_reg = nb_args as u16 + if ref_data.is_static { 0 } else { 1 };
    }

    let abort_label = format!(
        "end_static_call_to_{}_at_{:08X}",
        ref_data.method.try_to_smali()?,
        ref_data.addr
    );
    let mut insns = test_method(
        method_obj,
        ref_data.method.clone(),
        abort_label.clone(),
        reg_inf,
        tester_methods_class,
        tester_methods,
    )?;

    if !ref_data.is_static {
        // Move 'this' to fist arg
        // We do a small detour to `reg_inf.array_val` because we need a u8 reg to down cast the
        // Object reference to the right Class
        insns.push(Instruction::MoveObject {
            from: obj_inst,
            to: reg_inf.array_val as u16,
        });
        insns.push(Instruction::CheckCast {
            reg: reg_inf.array_val,
            lit: ref_data.method.class_.clone(),
        });
        insns.push(Instruction::MoveObject {
            from: reg_inf.array_val as u16,
            to: reg_inf.first_arg,
        });
    }
    insns.append(&mut get_args_from_obj_arr(
        &ref_data.method.proto.get_parameters(),
        arg_arr,
        reg_inf.first_arg + if ref_data.is_static { 0 } else { 1 },
        reg_inf,
    ));
    if ref_data.is_static {
        insns.push(Instruction::InvokeStatic {
            method: ref_data.method.clone(),
            args: (reg_inf.first_arg..reg_inf.first_arg + nb_args as u16).collect(),
        });
    } else {
        insns.push(Instruction::InvokeVirtual {
            method: ref_data.method.clone(),
            args: (reg_inf.first_arg..reg_inf.first_arg + 1 + nb_args as u16).collect(),
        });
    }
    if let Some(move_result) = move_result {
        let ret_ty = ref_data.method.proto.get_return_type();
        let res_reg = if let Instruction::MoveResultObject { to } = &move_result {
            *to
        } else {
            panic!(
                "`move_result` shloud always be a MoveResultObject, found {}",
                move_result.__str__()
            )
        };
        if ret_ty.is_class() || ret_ty.is_array() {
            insns.push(move_result);
        } else if ret_ty.is_double() || ret_ty.is_long() {
            insns.push(Instruction::MoveResultWide {
                to: reg_inf.array_val,
            });
            insns.push(Instruction::InvokeStatic {
                method: get_scalar_to_obj_method(&ret_ty).unwrap(),
                args: vec![reg_inf.array_val as u16],
            });
            insns.push(move_result);
            insns.push(Instruction::CheckCast {
                reg: res_reg,
                lit: OBJECT_TY.clone(),
            });
        } else {
            insns.push(Instruction::MoveResult {
                to: reg_inf.array_val,
            });
            insns.push(Instruction::InvokeStatic {
                method: get_scalar_to_obj_method(&ret_ty).unwrap(),
                args: vec![reg_inf.array_val as u16],
            });
            insns.push(move_result);
            insns.push(Instruction::CheckCast {
                reg: res_reg,
                lit: OBJECT_TY.clone(),
            });
        }
    }
    insns.push(Instruction::Goto {
        label: end_label.to_string(),
    });
    insns.push(Instruction::Label { name: abort_label });
    // We need a few u8 regs here. For now, we assumes we work with less than 256 reg.
    Ok(insns)
}

/// Generate bytecode that put the arguments of types `params` from an [java.lang.Object to
/// types consecutive registers starting at `first_arg_reg`.
/// `first_arg_reg` sould be `reg_inf.first_arg` or `reg_inf.first_arg+1` depending on if this
/// is for a static or virtual call.
fn get_args_from_obj_arr(
    params: &[IdType],
    array_reg: u16,
    first_arg_reg: u16,
    reg_inf: &mut RegistersInfo,
) -> Vec<Instruction> {
    let mut insns = vec![];
    let mut restore_array = vec![];
    let mut reg_count = 0;
    let array_reg = if array_reg <= 0b1111 {
        array_reg as u8
    } else {
        insns.push(Instruction::MoveObject {
            from: array_reg,
            to: reg_inf.array as u16,
        });
        restore_array.push(Instruction::MoveObject {
            from: reg_inf.array as u16,
            to: array_reg,
        });
        reg_inf.array
    };
    for (i, param) in params.iter().enumerate() {
        insns.push(Instruction::Const {
            reg: reg_inf.array_index,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: reg_inf.array_val,
            arr: array_reg,
            idx: reg_inf.array_index,
        });
        if param.is_class() || param.is_array() {
            insns.push(Instruction::CheckCast {
                reg: reg_inf.array_val,
                lit: param.clone(),
            });
            insns.push(Instruction::MoveObject {
                from: reg_inf.array_val as u16,
                to: first_arg_reg + reg_count,
            });
            reg_count += 1;
        } else if param.is_double() || param.is_long() {
            insns.push(Instruction::CheckCast {
                reg: reg_inf.array_val,
                lit: get_obj_of_scalar(param).unwrap(),
            });
            insns.push(Instruction::InvokeVirtual {
                method: get_obj_to_scalar_method(param).unwrap(),
                args: vec![reg_inf.array_val as u16],
            });
            insns.push(Instruction::MoveResultWide {
                to: reg_inf.array_val,
            });
            insns.push(Instruction::MoveWide {
                from: reg_inf.array_val as u16,
                to: first_arg_reg + reg_count,
            });
            reg_count += 2;
        } else {
            insns.push(Instruction::CheckCast {
                reg: reg_inf.array_val,
                lit: get_obj_of_scalar(param).unwrap(),
            });
            insns.push(Instruction::InvokeVirtual {
                method: get_obj_to_scalar_method(param).unwrap(),
                args: vec![reg_inf.array_val as u16],
            });
            insns.push(Instruction::MoveResult {
                to: reg_inf.array_val,
            });
            insns.push(Instruction::Move {
                from: reg_inf.array_val as u16,
                to: first_arg_reg + reg_count,
            });
            reg_count += 1;
        }
    }
    insns.append(&mut restore_array);
    insns
}

fn get_cnstr_new_inst_block(
    ref_data: &ReflectionCnstrNewInstData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
    end_label: &str,
    move_result: Option<Instruction>,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<IdMethod, Method>,
) -> Result<Vec<Instruction>> {
    let (cnst_reg, arg_arr) = if let &[a, b] = invoke_arg {
        (a, b)
    } else {
        bail!(
            "Method;->invoke arg should have exactrly 2 arguments, found {}",
            invoke_arg.len()
        );
    };

    let nb_args = ref_data.constructor.proto.get_parameters().len();
    if reg_inf.nb_arg_reg < nb_args as u16 + 1 {
        reg_inf.nb_arg_reg = nb_args as u16 + 1;
    }

    let abort_label = format!(
        "end_static_instance_with_{}_at_{}",
        ref_data.constructor.try_to_smali()?,
        "TODO_ADDR"
    );

    let mut insns = test_cnstr(
        cnst_reg,
        ref_data.constructor.clone(),
        abort_label.clone(),
        reg_inf,
        tester_methods_class,
        tester_methods,
    )?;
    insns.append(&mut get_args_from_obj_arr(
        &ref_data.constructor.proto.get_parameters(),
        arg_arr,
        reg_inf.first_arg + 1,
        reg_inf,
    ));
    if reg_inf.first_arg < u8::MAX as u16 {
        insns.push(Instruction::NewInstance {
            reg: reg_inf.first_arg as u8,
            lit: ref_data.constructor.class_.clone(),
        });
    } else {
        insns.push(Instruction::NewInstance {
            reg: reg_inf.array_val,
            lit: ref_data.constructor.class_.clone(),
        });
        insns.push(Instruction::MoveObject {
            from: reg_inf.array_val as u16,
            to: reg_inf.first_arg,
        });
    }
    insns.push(Instruction::InvokeDirect {
        method: ref_data.constructor.clone(),
        args: (reg_inf.first_arg..reg_inf.first_arg + nb_args as u16 + 1).collect(),
    });
    if let Some(Instruction::MoveResultObject { to }) = move_result {
        insns.push(Instruction::MoveObject {
            from: reg_inf.first_arg,
            to: to as u16,
        });
    }
    insns.push(Instruction::Goto {
        label: end_label.to_string(),
    });
    insns.push(Instruction::Label { name: abort_label });
    Ok(insns)
}
/// Generate bytecode that test if a `java.lang.reflect.Constructor` is equal to an [`IdMethod`]
///
/// - `method_obj_reg`: the register containing the `java.lang.reflect.Method`
/// - `id_method`: the expected [`IdMethod`].
/// - `abort_label`: the label where to jump if the method does not match `id_method`.
fn test_cnstr(
    cnst_reg: u16,
    id_method: IdMethod,
    abort_label: String,
    reg_inf: &mut RegistersInfo,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<IdMethod, Method>,
) -> Result<Vec<Instruction>> {
    use std::collections::hash_map::Entry;
    let tst_descriptor = match tester_methods.entry(id_method.clone()) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => e.insert(gen_tester_method(tester_methods_class, id_method, true)?),
    }
    .descriptor
    .clone();
    Ok(vec![
        Instruction::InvokeStatic {
            method: tst_descriptor,
            args: vec![cnst_reg],
        },
        Instruction::MoveResult {
            to: reg_inf.array_val,
        },
        Instruction::IfEqZ {
            a: reg_inf.array_val,
            label: abort_label,
        },
    ])
}

fn get_class_new_inst_block(
    ref_data: &ReflectionClassNewInstData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
    end_label: &str,
    move_result: Option<Instruction>,
) -> Result<Vec<Instruction>> {
    let class_reg = if let &[a] = invoke_arg {
        a
    } else {
        bail!(
            "Method;->invoke arg should have exactrly 3 arguments, found {}",
            invoke_arg.len()
        );
    };
    if !ref_data.constructor.proto.get_parameters().is_empty() {
        bail!(
            "Class.newInstance can only initialize instance with zero args constructor, found {}",
            ref_data.constructor.__str__()
        );
    }

    if class_reg > u8::MAX as u16 {
        // TODO
        bail!("Cannot transform instantiation calls to a class stored in 16 bits register");
    }
    let class_reg = class_reg as u8;

    let abort_label = format!(
        "end_static_instance_with_{}_at_{}",
        ref_data.constructor.try_to_smali()?,
        "TODO_ADDR"
    );

    let obj_reg = match move_result {
        Some(Instruction::MoveResultObject { to }) => to,
        _ => reg_inf.array_index,
    };

    Ok(vec![
        Instruction::ConstClass {
            reg: reg_inf.array_index, // wrong name, but available for tmp val
            lit: ref_data.constructor.class_.clone(),
        },
        Instruction::IfNe {
            a: reg_inf.array_index,
            b: class_reg,
            label: abort_label.clone(),
        },
        Instruction::NewInstance {
            reg: obj_reg,
            lit: ref_data.constructor.class_.clone(),
        },
        Instruction::InvokeDirect {
            method: ref_data.constructor.clone(),
            args: vec![obj_reg as u16],
        },
        Instruction::Goto {
            label: end_label.to_string(),
        },
        Instruction::Label { name: abort_label },
    ])
}
