use androscalpel::SmaliName;
use androscalpel::{Code, IdMethod, IdMethodType, IdType, Instruction, Method};
use anyhow::{bail, Context, Result};
use log::{debug, warn};

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{dex_types::*, register_manipulation::*, runtime_data::*};

const DEBUG: bool = true;

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
    runtime_data: &RuntimeData,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<(IdMethod, String), Method>,
) -> Result<()> {
    // checking meth.annotations might be usefull at some point
    //println!("{}", meth.descriptor.__str__());
    let invoke_data = runtime_data.get_invoke_data_for(&meth.descriptor);
    let class_new_inst_data = runtime_data.get_class_new_instance_data_for(&meth.descriptor);
    let cnstr_new_inst_data = runtime_data.get_cnstr_new_instance_data_for(&meth.descriptor);

    let code = meth
        .code
        .as_ref()
        .with_context(|| format!("Code not found in {}", meth.descriptor.__str__()))?;

    // Get the available registers at the method level
    let mut register_info = RegistersInfo::default();
    debug!("Pathching method {}", meth.__str__());
    // register_info.array_val is a wide reg, so need at least 0b1110 and 0b1111
    if code.registers_size < 0b1111 {
        register_info.array_val = code.registers_size as u8;
        debug!(
            "Use registers {}-{} for patching",
            register_info.array_val,
            register_info.array_val + 1
        );
    } else {
        register_info.array_val = 0;
        register_info.array_val_save = Some(code.registers_size);
        debug!(
            "Too many registers, reserve registers {}-{} to save registers later on",
            code.registers_size,
            code.registers_size + 1
        );
    }
    if code.registers_size + 2 <= 0b1111 {
        register_info.array_index = (code.registers_size + 2) as u8;
        debug!("Use register {} for patching", register_info.array_index);
    } else {
        register_info.array_index = 0;
        register_info.array_index_save = Some(code.registers_size + 2);
        debug!(
            "Too many registers, reserve register {} to save registers later on",
            code.registers_size + 2
        );
    }
    if code.registers_size + 3 <= 0b1111 {
        register_info.array = (code.registers_size + 3) as u8;
        debug!("Use register {} for patching", register_info.array);
    } else {
        register_info.array = 0;
        register_info.array_save = Some(code.registers_size + 3);
        debug!(
            "Too many registers, reserve register {} to save registers later on",
            code.registers_size + 3
        );
    }
    register_info.first_arg = code.registers_size + 4;
    debug!(
        "Will use register from {} on to store method arguments",
        register_info.first_arg
    );
    register_info.nb_arg_reg = 0; // Will be set when saving args

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
            'invoke_patch: {
                let addr_label = current_addr_label.as_ref().unwrap();
                debug!(
                    "Patching reflection call of {} at {}:{}",
                    method.__str__(),
                    meth.__str__(),
                    addr_label
                );
                let end_label = if method == &*MTH_INVOKE {
                    format!("end_reflection_call_at_{}", addr_label.clone())
                } else if method == &*CLASS_NEW_INST || method == &*CNSTR_NEW_INST {
                    format!("end_reflection_instanciation_at_{}", addr_label.clone())
                } else {
                    // This should not happen, cf the guard on the match
                    warn!(
                        "Reflection Data point to an invoke-virtual {}, (expected invocation of {}, {} or {})",
                        method.__str__(),
                        MTH_INVOKE.__str__(),
                        CLASS_NEW_INST.__str__(),
                        CNSTR_NEW_INST.__str__()
                    );
                    new_insns.push(ins.clone());
                    break 'invoke_patch;
                };

                let (pseudo_insns, move_ret) = get_move_result(iter.clone());
                if move_ret.is_some() {
                    while move_ret.as_ref() != iter.next() {}
                }

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
                                break 'invoke_patch;
                            }
                        }
                    }
                }
                // TODO: recover from failure
                if method == &*MTH_INVOKE {
                    for ref_data in invoke_data.get(addr_label).unwrap_or(&vec![]) {
                        debug!(
                            "Patching reflection call at {}:{} to {}",
                            meth.descriptor.__str__(),
                            addr_label,
                            ref_data.method.__str__()
                        );
                        for ins in get_invoke_block(
                            ref_data,
                            args.as_slice(),
                            &mut register_info,
                            &end_label,
                            move_ret.clone(),
                            tester_methods_class.clone(),
                            tester_methods,
                            runtime_data,
                        )? {
                            new_insns.push(ins);
                        }
                    }
                } else if method == &*CLASS_NEW_INST {
                    for ref_data in class_new_inst_data.get(addr_label).unwrap_or(&vec![]) {
                        debug!(
                            "Patching reflection instantion at {}:{} for {}",
                            meth.descriptor.__str__(),
                            addr_label,
                            ref_data.constructor.__str__()
                        );
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
                        debug!(
                            "Patching reflection instantion at {}:{} for {}",
                            meth.descriptor.__str__(),
                            addr_label,
                            ref_data.constructor.__str__()
                        );
                        if ref_data.constructor
                            == IdMethod::from_smali(
                                "Lcom/example/theseus/dynandref/AReflectee;-><init>()V",
                            )
                            .unwrap()
                            &&
                            meth.descriptor == IdMethod::from_smali(
                                "Lcom/example/theseus/dynandref/Main;->factoryInterface(Landroid/app/Activity;Ljava/lang/Class;ZBSCIJFD[Ljava/lang/String;)V"
                            ).unwrap()
                        {
                            let mut cl_id = Some(&ref_data.constructor_cl_id);
                            while let Some(id) = cl_id {
                                let cl = runtime_data.classloaders.get(id);
                                if let Some(cl) = cl {
                                    cl_id = cl.parent_id.as_ref();
                                } else {
                                    cl_id = None
                                };
                            }
                        }
                        for ins in get_cnstr_new_inst_block(
                            ref_data,
                            args.as_slice(),
                            &mut register_info,
                            &end_label,
                            move_ret.clone(),
                            tester_methods_class.clone(),
                            tester_methods,
                            runtime_data,
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
        let new_arg_reg = code.registers_size - ins_size + i + register_info.get_nb_added_reg();
        let old_arg_reg = code.registers_size - ins_size + i;
        debug!(
            "Move `this` argument from new argument register {} to original register {}",
            new_arg_reg, old_arg_reg
        );
        code.insns.push(Instruction::MoveObject {
            from: new_arg_reg,
            to: old_arg_reg,
        });
        i += 1;
    }
    for arg in &meth.descriptor.proto.get_parameters() {
        let new_arg_reg = code.registers_size - ins_size + i + register_info.get_nb_added_reg();
        let old_arg_reg = code.registers_size - ins_size + i;
        if arg.is_class() || arg.is_array() {
            code.insns.push(Instruction::MoveObject {
                from: new_arg_reg,
                to: old_arg_reg,
            });
            debug!(
                "Move reference argument from new argument register {} to original register {}",
                new_arg_reg, old_arg_reg
            );
            i += 1;
        } else if arg.is_long() || arg.is_double() {
            code.insns.push(Instruction::MoveWide {
                from: new_arg_reg,
                to: old_arg_reg,
            });
            debug!(
                "Move wide argument from new argument registers {}-{} to original registers {}-{}",
                new_arg_reg,
                new_arg_reg + 1,
                old_arg_reg,
                new_arg_reg + 1
            );
            i += 2;
        } else {
            code.insns.push(Instruction::Move {
                from: new_arg_reg,
                to: old_arg_reg,
            });
            debug!(
                "Move scalar argument from new argument register {} to original register {}",
                new_arg_reg, old_arg_reg
            );
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
    classloader: Option<String>,
    _runtime_data: &RuntimeData,
) -> Result<Method> {
    let mut hasher = DefaultHasher::new();
    if let Some(ref id) = classloader {
        id.hash(&mut hasher);
    } else {
        "00000000".hash(&mut hasher);
    }
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

    let method_test_name = format!("check_is_{c_name}_{m_name}_{hash:016x}"); // hash depend on
                                                                              // classloader and
                                                                              // full method descr
    let descriptor = IdMethod::new(
        method_test_name.as_str().into(),
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
    let (
        no_label_wrong_number_of_arg,
        no_label_wrong_arg_type,
        no_label_wrong_meth_name,
        no_label_wrong_return_type,
        no_label_wrong_classloader_expected_bootclassloader,
        no_label_wrong_classloader_got_null,
        no_label_wrong_classloader,
        no_label_wrong_def_type,
    ) = if DEBUG {
        (
            "label_no_wrong_number_of_arg".into(),
            "label_no_wrong_arg_type".into(),
            "label_no_wrong_meth_name".into(),
            "label_no_wrong_return_type".into(),
            "label_no_wrong_classloader_expected_bootclassloader".into(),
            "label_no_wrong_classloader_got_null".into(),
            "label_no_wrong_classloader".into(),
            "label_no_wrong_def_type".into(),
        )
    } else {
        (
            no_label.clone(),
            no_label.clone(),
            no_label.clone(),
            no_label.clone(),
            no_label.clone(),
            no_label.clone(),
            no_label.clone(),
            no_label.clone(),
        )
    };
    const REG_ARR: u8 = 0;
    const REG_ARR_IDX: u8 = 1;
    const REG_TST_VAL: u8 = 2;
    const REG_DEF_TYPE: u8 = 3;
    const REG_CMP_VAL: u8 = 4;
    const _REG_CLASS_LOADER: u8 = 5;
    const REG_REGEX: u8 = 6;
    const REG_REPLACE: u8 = 7;
    const REG_IF_RES: u8 = 8;
    const REG_REF_METHOD: u8 = 9;

    /*
    fn standardize_name(reg: u8, old_app_path: &str) -> Vec<Instruction> {
        let tmp_reg = REG_IF_RES;
        vec![
            // Get the path of the current APK
            Instruction::InvokeStatic {
                method: GET_APP.clone(),
                args: vec![],
            },
            Instruction::MoveResultObject { to: tmp_reg },
            Instruction::InvokeVirtual {
                method: GET_APP_INFO.clone(),
                args: vec![tmp_reg as u16],
            },
            Instruction::MoveResultObject { to: tmp_reg },
            Instruction::IGetObject {
                to: tmp_reg,
                obj: tmp_reg,
                field: APP_INFO_SOURCE_DIR.clone(),
            },
            // Remove the "/base.apk" at the end of the path
            Instruction::ConstString {
                reg: REG_REGEX,
                lit: "/base\\.apk$".into(),
            },
            Instruction::ConstString {
                reg: REG_REPLACE,
                lit: "".into(),
            },
            Instruction::InvokeVirtual {
                method: STRING_REPLACE_ALL.clone(),
                args: vec![tmp_reg as u16, REG_REGEX as u16, REG_REPLACE as u16],
            },
            Instruction::MoveResultObject { to: REG_REGEX },
            // replace current app path in name
            Instruction::ConstString {
                reg: REG_REPLACE,
                lit: "APP_PATH".into(),
            },
            Instruction::InvokeVirtual {
                method: STRING_REPLACE_ALL.clone(),
                args: vec![reg as u16, REG_REGEX as u16, REG_REPLACE as u16],
            },
            Instruction::MoveResultObject { to: reg },
            // replace the old app path in name
            Instruction::ConstString {
                reg: REG_REGEX,
                lit: old_app_path.into(),
            },
            Instruction::InvokeVirtual {
                method: STRING_REPLACE_ALL.clone(),
                args: vec![reg as u16, REG_REGEX as u16, REG_REPLACE as u16],
            },
            Instruction::MoveResultObject { to: reg },
            // remove the in memory cookie parameters (change from one run to another)
            Instruction::ConstString {
                reg: REG_REGEX,
                lit: "InMemoryDexFile\\[cookie=\\[\\d*, \\d*\\]\\]".into(),
            },
            Instruction::ConstString {
                reg: REG_REPLACE,
                lit: "InMemoryDexFile".into(),
            },
            Instruction::InvokeVirtual {
                method: STRING_REPLACE_ALL.clone(),
                args: vec![reg as u16, REG_REGEX as u16, REG_REPLACE as u16],
            },
            Instruction::MoveResultObject { to: reg },
        ]
    }*/

    // Check for arg type
    let mut insns = if !is_constructor {
        vec![
            Instruction::InvokeVirtual {
                method: MTH_GET_PARAMS_TY.clone(),
                args: vec![REG_REF_METHOD as u16],
            },
            Instruction::MoveResultObject { to: REG_ARR },
        ]
    } else {
        vec![
            Instruction::InvokeVirtual {
                method: CNSTR_GET_PARAMS_TY.clone(),
                args: vec![REG_REF_METHOD as u16],
            },
            Instruction::MoveResultObject { to: REG_ARR },
        ]
    };
    // First check  the number of args
    // --------------------
    insns.append(&mut vec![
        Instruction::ArrayLength {
            dest: REG_ARR_IDX,
            arr: REG_ARR,
        },
        Instruction::Const {
            reg: REG_TST_VAL,
            lit: method_to_test.proto.get_parameters().len() as i32,
        },
        Instruction::IfNe {
            a: REG_ARR_IDX,
            b: REG_TST_VAL,
            label: no_label_wrong_number_of_arg.clone(),
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
            reg: REG_ARR_IDX,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: REG_TST_VAL,
            arr: REG_ARR,
            idx: REG_ARR_IDX,
        });
        insns.push(Instruction::ConstClass {
            reg: REG_CMP_VAL,
            lit: param,
        });
        insns.push(Instruction::InvokeVirtual {
            method: CLT_GET_DESCR_STRING.clone(),
            args: vec![REG_CMP_VAL as u16],
        });
        insns.push(Instruction::MoveResultObject { to: REG_CMP_VAL });
        insns.push(Instruction::InvokeVirtual {
            method: CLT_GET_DESCR_STRING.clone(),
            args: vec![REG_TST_VAL as u16],
        });
        insns.push(Instruction::MoveResultObject { to: REG_TST_VAL });
        insns.push(Instruction::InvokeVirtual {
            method: STR_EQ.clone(),
            args: vec![REG_CMP_VAL as u16, REG_TST_VAL as u16],
        });
        insns.push(Instruction::MoveResult { to: REG_IF_RES });
        insns.push(Instruction::IfEqZ {
            a: REG_IF_RES,
            label: no_label_wrong_arg_type.clone(),
        });
        // Comparing Type does not work when different types share the same name (eg type from
        // another class loader)
        //insns.push(Instruction::IfNe {
        //    a: REG_ARR_IDX,
        //    b: REG_TST_VAL,
        //  label: no_label.clone(),
        //})
    }
    if !is_constructor {
        insns.append(&mut vec![
            // Check the runtime method is the right one
            // Check Name
            Instruction::InvokeVirtual {
                method: MTH_GET_NAME.clone(),
                args: vec![REG_REF_METHOD as u16],
            },
            Instruction::MoveResultObject { to: REG_TST_VAL },
            Instruction::ConstString {
                reg: REG_CMP_VAL,
                lit: method_to_test.name.clone(),
            },
            Instruction::InvokeVirtual {
                method: STR_EQ.clone(),
                args: vec![REG_TST_VAL as u16, REG_CMP_VAL as u16],
            },
            Instruction::MoveResult { to: REG_IF_RES },
            Instruction::IfEqZ {
                a: REG_IF_RES,
                label: no_label_wrong_meth_name.clone(),
            },
            // Check Return Type
            Instruction::InvokeVirtual {
                method: MTH_GET_RET_TY.clone(),
                args: vec![REG_REF_METHOD as u16],
            },
            Instruction::MoveResultObject { to: REG_TST_VAL },
            Instruction::InvokeVirtual {
                method: CLT_GET_DESCR_STRING.clone(),
                args: vec![REG_TST_VAL as u16],
            },
            Instruction::MoveResultObject { to: REG_TST_VAL },
            Instruction::ConstClass {
                reg: REG_CMP_VAL,
                lit: method_to_test.proto.get_return_type(),
            },
            Instruction::InvokeVirtual {
                method: CLT_GET_DESCR_STRING.clone(),
                args: vec![REG_CMP_VAL as u16],
            },
            Instruction::MoveResultObject { to: REG_CMP_VAL },
            Instruction::InvokeVirtual {
                method: STR_EQ.clone(),
                args: vec![REG_CMP_VAL as u16, REG_TST_VAL as u16],
            },
            Instruction::MoveResult { to: REG_IF_RES },
            Instruction::IfEqZ {
                a: REG_IF_RES,
                label: no_label_wrong_return_type.clone(),
            },
            // Comparing Type does not work when different types share the same name (eg type from
            // another class loader)
            //Instruction::IfNe {
            //    a: REG_ARR_IDX,
            //    b: REG_TST_VAL,
            //    label: no_label.clone(),
            //},
        ]);
    }
    // Get Declaring Type
    if is_constructor {
        insns.push(Instruction::InvokeVirtual {
            method: CNSTR_GET_DEC_CLS.clone(),
            args: vec![REG_REF_METHOD as u16],
        });
    } else {
        insns.push(Instruction::InvokeVirtual {
            method: MTH_GET_DEC_CLS.clone(),
            args: vec![REG_REF_METHOD as u16],
        });
    }
    insns.push(Instruction::MoveResultObject { to: REG_DEF_TYPE });

    /*  Checking classloader is complicated: adding the classes to the appliction change the
     *  behavior of classloader, so this tst wont work. To make this work, all classes reinjected
     *  to the application would need to be renammed.
    //Check the classloader
    let mut current_classloader = classloader
        .as_ref()
        .and_then(|id| runtime_data.classloaders.get(id));
    let check_class_loader = current_classloader.is_some();
    if check_class_loader {
        insns.append(&mut vec![
            // Get the string representation of the classloader.
            // Not the ideal, but best cross execution classloader identifier we have.
            Instruction::InvokeVirtual {
                method: GET_CLASS_LOADER.clone(),
                args: vec![REG_DEF_TYPE as u16],
            },
            Instruction::MoveResultObject {
                to: REG_CLASS_LOADER,
            },
        ]);
    }
    while let Some(classloader) = current_classloader {
        // TODO: check class and if platform
        if classloader.cname == *BOOT_CLASS_LOADER_TY {
            // Ljava/lang/BootClassLoader; is complicated.
            // It's string rep is "java.lang.BootClassLoader@7e2aeab" where "7e2aeab" is it's
            // runtime hash id: the name change at each run. We need to compare with its type (it's
            // ok, it's supposed to be a singleton).
            // Also, it can be represented at runtime by the null pointer, so we need to accept the
            // null pointer as a valid value.
            insns.append(&mut vec![
                Instruction::IfEqZ {
                    a: REG_CLASS_LOADER,
                    label: "label_end_classloader_test".into(),
                },
                Instruction::InvokeVirtual {
                    method: GET_CLASS.clone(),
                    args: vec![REG_CLASS_LOADER as u16],
                },
                Instruction::MoveResultObject { to: REG_TST_VAL },
                Instruction::InvokeVirtual {
                    method: CLT_GET_DESCR_STRING.clone(),
                    args: vec![REG_TST_VAL as u16],
                },
                Instruction::MoveResultObject { to: REG_TST_VAL },
                /* Illegal class access
                Instruction::ConstClass {
                    reg: REG_CMP_VAL,
                    lit: BOOT_CLASS_LOADER_TY.clone(),
                },
                Instruction::InvokeVirtual {
                    method: CLT_GET_DESCR_STRING.clone(),
                    args: vec![REG_CMP_VAL as u16],
                },
                Instruction::MoveResultObject { to: REG_CMP_VAL },
                */
                Instruction::ConstString {
                    reg: REG_CMP_VAL,
                    lit: "Ljava/lang/BootClassLoader;".into(), // why not doted repr? android? why?
                },
                Instruction::InvokeVirtual {
                    method: STR_EQ.clone(),
                    args: vec![REG_CMP_VAL as u16, REG_TST_VAL as u16],
                },
                Instruction::MoveResult { to: REG_IF_RES },
                Instruction::IfEqZ {
                    a: REG_IF_RES,
                    label: no_label_wrong_classloader_expected_bootclassloader.clone(),
                },
                Instruction::Label {
                    name: "label_end_classloader_test".into(),
                },
            ]);
            break;
        }
        insns.append(&mut vec![
            Instruction::IfEqZ {
                a: REG_CLASS_LOADER,
                label: no_label_wrong_classloader_got_null.clone(),
            },
            Instruction::InvokeVirtual {
                method: TO_STRING.clone(),
                args: vec![REG_CLASS_LOADER as u16],
            },
            Instruction::MoveResultObject { to: REG_TST_VAL },
            Instruction::ConstString {
                reg: REG_CMP_VAL,
                lit: classloader.string_representation.as_str().into(),
            },
        ]);
        insns.append(&mut standardize_name(
            REG_CMP_VAL,
            &runtime_data.app_info.actual_source_dir,
        ));
        insns.append(&mut standardize_name(
            REG_TST_VAL,
            &runtime_data.app_info.actual_source_dir,
        ));
        insns.append(&mut vec![
            Instruction::InvokeVirtual {
                method: STR_EQ.clone(),
                args: vec![REG_CMP_VAL as u16, REG_TST_VAL as u16],
            },
            Instruction::MoveResult { to: REG_IF_RES },
            Instruction::IfEqZ {
                a: REG_IF_RES,
                label: no_label_wrong_classloader.clone(),
            },
            Instruction::InvokeVirtual {
                method: GET_PARENT.clone(),
                args: vec![REG_CLASS_LOADER as u16],
            },
            Instruction::MoveResultObject {
                to: REG_CLASS_LOADER,
            },
        ]);
        let parent_id = classloader.parent_id.clone();
        // If parent_id is None, the parent is in fact the boot class loader (except for the
        // boot class loader itself, already handled at the start of the loop).
        current_classloader = if let Some(ref id) = parent_id {
            runtime_data.classloaders.get(id)
        } else {
            runtime_data
                .classloaders
                .values()
                .find(|cl| cl.cname == *BOOT_CLASS_LOADER_TY)
        };
    }
    */

    // Check Declaring Type
    insns.append(&mut vec![
        Instruction::ConstClass {
            reg: REG_CMP_VAL,
            lit: method_to_test.class_.clone(),
        },
        Instruction::InvokeVirtual {
            method: CLT_GET_DESCR_STRING.clone(),
            args: vec![REG_CMP_VAL as u16],
        },
        Instruction::MoveResultObject { to: REG_CMP_VAL },
        Instruction::InvokeVirtual {
            method: CLT_GET_DESCR_STRING.clone(),
            args: vec![REG_DEF_TYPE as u16],
        },
        Instruction::MoveResultObject { to: REG_TST_VAL },
        Instruction::InvokeVirtual {
            method: STR_EQ.clone(),
            args: vec![REG_CMP_VAL as u16, REG_TST_VAL as u16],
        },
        Instruction::MoveResult { to: REG_IF_RES },
        Instruction::IfEqZ {
            a: REG_IF_RES,
            label: no_label_wrong_def_type.clone(),
        },
        // Comparing Type does not work when different types share the same name (eg type from
        // another class loader)
        //Instruction::IfNe {
        //    a: REG_ARR_IDX,
        //    b: REG_TST_VAL,
        //    label: no_label.clone(),
        //},
    ]);
    if DEBUG {
        insns.append(&mut vec![
            Instruction::ConstString {
                reg: REG_TST_VAL,
                lit: "THESEUS".into(),
            },
            Instruction::ConstString {
                reg: REG_CMP_VAL,
                lit: format!(
                    "T.{method_test_name}() (test of {}) returned true",
                    method_to_test
                        .try_to_smali()
                        .unwrap_or("failed to convert".into())
                )
                .into(),
            },
            Instruction::InvokeStatic {
                method: LOG_INFO.clone(),
                args: vec![REG_TST_VAL as u16, REG_CMP_VAL as u16],
            },
        ]);
    }
    insns.append(&mut vec![
        Instruction::Const {
            reg: REG_CMP_VAL,
            lit: 1,
        },
        Instruction::Return { reg: REG_CMP_VAL },
    ]);
    if DEBUG {
        for label_name in &[
            &no_label_wrong_number_of_arg,
            &no_label_wrong_arg_type,
            &no_label_wrong_meth_name,
            &no_label_wrong_return_type,
            &no_label_wrong_classloader_expected_bootclassloader,
            &no_label_wrong_classloader_got_null,
            &no_label_wrong_classloader,
            &no_label_wrong_def_type,
        ] {
            let reg_tag = REG_REGEX;
            let reg_msg = REG_REPLACE;
            insns.push(Instruction::Label {
                name: (*label_name).clone(),
            });
            insns.push(Instruction::ConstString {
                reg: reg_tag,
                lit: "THESEUS".into(),
            });
            insns.push(Instruction::ConstString {
                reg: reg_msg,
                lit: format!(
                    "T.{method_test_name}() (test of {}) returned false",
                    method_to_test
                        .try_to_smali()
                        .unwrap_or("failed to convert".into())
                )
                .into(),
            });
            insns.push(Instruction::InvokeStatic {
                method: LOG_INFO.clone(),
                args: vec![reg_tag as u16, reg_msg as u16],
            });
            if label_name == &&no_label_wrong_number_of_arg {
                insns.push(Instruction::ConstString {
                    reg: reg_msg,
                    lit: "Wrong number of arg".into(),
                });
            } else if label_name == &&no_label_wrong_arg_type {
                insns.push(Instruction::ConstString {
                    reg: reg_msg,
                    lit: "Wrong type of arg".into(),
                });
            } else if label_name == &&no_label_wrong_meth_name {
                insns.push(Instruction::ConstString {
                    reg: reg_msg,
                    lit: "Wrong method name".into(),
                });
            } else if label_name == &&no_label_wrong_return_type {
                insns.push(Instruction::ConstString {
                    reg: reg_msg,
                    lit: "Wrong return type".into(),
                });
            } else if label_name == &&no_label_wrong_classloader_expected_bootclassloader {
                insns.append(&mut vec![
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Wrong classloader, expected bootclassloader, got: ".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, REG_TST_VAL as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "".into(),
                    },
                ]);
            } else if label_name == &&no_label_wrong_classloader_got_null {
                insns.push(Instruction::ConstString {
                    reg: reg_msg,
                    lit: "Wrong classloader, got null instead of object".into(),
                });
            } else if label_name == &&no_label_wrong_classloader {
                insns.append(&mut vec![
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Wrong classloader".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Expected: ".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, REG_CMP_VAL as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Got: ".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, REG_TST_VAL as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "".into(),
                    },
                ]);
            } else if label_name == &&no_label_wrong_def_type {
                insns.append(&mut vec![
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Wrong class".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Expected: ".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, REG_CMP_VAL as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "Got: ".into(),
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, reg_msg as u16],
                    },
                    Instruction::InvokeStatic {
                        method: LOG_INFO.clone(),
                        args: vec![reg_tag as u16, REG_TST_VAL as u16],
                    },
                    Instruction::ConstString {
                        reg: reg_msg,
                        lit: "".into(),
                    },
                ]);
            }

            insns.append(&mut vec![
                Instruction::InvokeStatic {
                    method: LOG_INFO.clone(),
                    args: vec![reg_tag as u16, reg_msg as u16],
                },
                Instruction::Const {
                    reg: REG_CMP_VAL,
                    lit: 0,
                },
                Instruction::Return { reg: REG_CMP_VAL },
            ]);
        }
    } else {
        insns.append(&mut vec![
            Instruction::Label { name: no_label },
            Instruction::Const {
                reg: REG_CMP_VAL,
                lit: 0,
            },
            Instruction::Return { reg: REG_CMP_VAL },
        ]);
    }

    method.is_static = true;
    method.is_final = true;
    method.code = Some(Code::new(
        10, //registers_size, 9 reg + 1 parameter reg
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
///   Methods are indexed by the IdMethod they detect, and have a name derived from the method
///   they detect.
/// - `classloader`: is the runtime data of the classloader that loaded the class defining the
///   reflected method. If None, the classloader is not tested. Platform classes should probably
///   not be tested (the bootclassloader can be represented with a null reference, which may
///   lead to a null pointer exception).
#[allow(clippy::too_many_arguments)]
fn test_method(
    method_obj_reg: u16,
    id_method: IdMethod,
    abort_label: String,
    reg_inf: &mut RegistersInfo,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<(IdMethod, String), Method>,
    classloader: Option<String>,
    runtime_data: &RuntimeData,
) -> Result<Vec<Instruction>> {
    use std::collections::hash_map::Entry;
    let key = (id_method.clone(), classloader.clone().unwrap_or("".into()));
    let tst_descriptor = match tester_methods.entry(key) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => e.insert(gen_tester_method(
            tester_methods_class,
            id_method,
            false,
            classloader,
            runtime_data,
        )?),
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

#[allow(clippy::too_many_arguments)]
fn get_invoke_block(
    ref_data: &ReflectionInvokeData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
    end_label: &str,
    move_result: Option<Instruction>,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<(IdMethod, String), Method>,
    runtime_data: &RuntimeData,
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

    let abort_label = {
        // method descriptor in label are hard to debug
        let name = format!(
            "end_static_call_to_{}_from_classloader_{}_at_{:08X}",
            ref_data.method.try_to_smali()?,
            &ref_data.method_cl_id,
            ref_data.addr
        );
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("end_static_call_{:x}", hasher.finish())
    };
    let classloader = if ref_data.method.class_.is_platform_class() {
        None
    } else {
        Some(ref_data.method_cl_id.clone())
    };
    let mut insns = test_method(
        method_obj,
        ref_data.method.clone(),
        abort_label.clone(),
        reg_inf,
        tester_methods_class,
        tester_methods,
        classloader,
        runtime_data,
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
            lit: ref_data.get_static_callee().class_,
        });
        insns.push(Instruction::MoveObject {
            from: reg_inf.array_val as u16,
            to: reg_inf.first_arg,
        });
    }
    insns.append(&mut get_args_from_obj_arr(
        &ref_data.get_static_callee().proto.get_parameters(), // TODO: what if renambed args?
        arg_arr,
        reg_inf.first_arg + if ref_data.is_static { 0 } else { 1 },
        reg_inf,
    ));
    if ref_data.is_static {
        insns.push(Instruction::InvokeStatic {
            method: ref_data.get_static_callee(),
            args: (reg_inf.first_arg..reg_inf.first_arg + nb_args as u16).collect(),
        });
    } else {
        insns.push(Instruction::InvokeVirtual {
            method: ref_data.get_static_callee(),
            args: (reg_inf.first_arg..reg_inf.first_arg + 1 + nb_args as u16).collect(),
        });
    }
    if let Some(move_result) = move_result {
        let ret_ty = ref_data.get_static_callee().proto.get_return_type();
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

pub(crate) static ABORD_LABELS: std::sync::LazyLock<
    std::sync::Mutex<HashMap<String, Vec<ReflectionCnstrNewInstData>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

#[allow(clippy::too_many_arguments)]
fn get_cnstr_new_inst_block(
    ref_data: &ReflectionCnstrNewInstData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
    end_label: &str,
    move_result: Option<Instruction>,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<(IdMethod, String), Method>,
    runtime_data: &RuntimeData,
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

    let abort_label = {
        // method descriptor in label are hard to debug
        let name = format!(
            "end_static_instance_with_{}_from_classloader_{}_at_{:08X}",
            ref_data.constructor.try_to_smali()?,
            &ref_data.constructor_cl_id,
            ref_data.addr
        );
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("end_static_call_{:x}", hasher.finish())
    };
    if ref_data.caller_method == IdMethod::from_smali("Lcom/example/theseus/dynandref/Main;->factoryInterface(Landroid/app/Activity;Ljava/lang/Class;ZBSCIJFD[Ljava/lang/String;)V").unwrap() {
        ABORD_LABELS.lock().unwrap().entry(abort_label.clone()).or_default().push(ref_data.clone());
    }

    let classloader = if ref_data.constructor.class_.is_platform_class() {
        None
    } else {
        Some(ref_data.constructor_cl_id.clone())
    };
    let mut insns = test_cnstr(
        cnst_reg,
        ref_data.constructor.clone(), // TODO: what if args are renammed?
        abort_label.clone(),
        reg_inf,
        tester_methods_class,
        tester_methods,
        classloader,
        runtime_data,
    )?;
    insns.append(&mut get_args_from_obj_arr(
        &ref_data.constructor.proto.get_parameters(), // TODO: what if args are renammed?
        arg_arr,
        reg_inf.first_arg + 1,
        reg_inf,
    ));
    if reg_inf.first_arg < u8::MAX as u16 {
        insns.push(Instruction::NewInstance {
            reg: reg_inf.first_arg as u8,
            lit: ref_data.get_static_constructor().class_,
        });
    } else {
        insns.push(Instruction::NewInstance {
            reg: reg_inf.array_val,
            lit: ref_data.get_static_constructor().class_,
        });
        insns.push(Instruction::MoveObject {
            from: reg_inf.array_val as u16,
            to: reg_inf.first_arg,
        });
    }
    insns.push(Instruction::InvokeDirect {
        method: ref_data.get_static_constructor(),
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
/// - `tester_methods_class`: the class used to define the methods in `tester_methods`
/// - `tester_methods`: the methods used to test if a `java.lang.reflect.Method` is a specific method.
///   Methods are indexed by the IdMethod they detect, and have a name derived from the method
///   they detect.
/// - `classloader`: is the runtime data of the classloader that loaded the. If None, the classloader
///   is not tested. Platform classes should probably not be tested (the bootclassloader can be
///   represented with a null reference, which may lead to a null pointer exception).
#[allow(clippy::too_many_arguments)]
fn test_cnstr(
    cnst_reg: u16,
    id_method: IdMethod,
    abort_label: String,
    reg_inf: &mut RegistersInfo,
    tester_methods_class: IdType,
    tester_methods: &mut HashMap<(IdMethod, String), Method>,
    classloader: Option<String>,
    runtime_data: &RuntimeData,
) -> Result<Vec<Instruction>> {
    use std::collections::hash_map::Entry;
    let key = (id_method.clone(), classloader.clone().unwrap_or("".into()));
    let tst_descriptor = match tester_methods.entry(key) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => e.insert(gen_tester_method(
            tester_methods_class,
            id_method,
            true,
            classloader,
            runtime_data,
        )?),
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

    let class_reg = class_reg as u8;

    let abort_label = {
        // method descriptor in label are hard to debug
        let name = format!(
            "end_static_instance_with_{}_from_classloader_{}_at_{:08X}",
            ref_data.constructor.try_to_smali()?,
            &ref_data.constructor_cl_id,
            ref_data.addr
        );
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("end_static_call_{:x}", hasher.finish())
    };

    let obj_reg = match move_result {
        Some(Instruction::MoveResultObject { to }) => to,
        _ => reg_inf.array_index,
    };

    Ok(vec![
        Instruction::ConstClass {
            reg: reg_inf.array_index, // wrong name, but available for tmp val
            lit: ref_data.constructor.class_.clone(),
        },
        Instruction::InvokeVirtual {
            method: CLT_GET_DESCR_STRING.clone(),
            args: vec![reg_inf.array_index as u16],
        },
        Instruction::MoveResultObject {
            to: reg_inf.array_index,
        },
        Instruction::InvokeVirtual {
            method: CLT_GET_DESCR_STRING.clone(),
            args: vec![class_reg as u16],
        },
        Instruction::MoveResultObject { to: class_reg },
        Instruction::InvokeVirtual {
            method: STR_EQ.clone(),
            args: vec![reg_inf.array_index as u16, class_reg as u16],
        },
        Instruction::MoveResult {
            to: reg_inf.array_index, // wrong name, but available for tmp val
        },
        Instruction::IfEqZ {
            a: reg_inf.array_index,
            label: abort_label.clone(),
        },
        // Comparing Type does not work when different types share the same name (eg type from
        // another class loader)
        //Instruction::IfNe {
        //    a: reg_inf.array_index,
        //    b: class_reg,
        //    label: abort_label.clone(),
        //},
        Instruction::NewInstance {
            reg: obj_reg,
            lit: ref_data.get_static_constructor().class_.clone(),
        },
        Instruction::InvokeDirect {
            method: ref_data.get_static_constructor().clone(),
            args: vec![obj_reg as u16],
        },
        Instruction::Goto {
            label: end_label.to_string(),
        },
        Instruction::Label { name: abort_label },
    ])
}
