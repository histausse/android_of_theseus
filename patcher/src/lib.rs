use androscalpel::SmaliName;
use androscalpel::{IdMethod, Instruction, Method};
use anyhow::{bail, Context, Result};
use log::warn;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

// TODO:
// Check what
// https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/reflection.cc;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=698
// does.

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ReflectionData {
    pub invoke_data: Vec<ReflectionInvokeData>,
    pub class_new_inst_data: Vec<ReflectionClassNewInstData>,
    pub cnstr_new_inst_data: Vec<ReflectionCnstrNewInstData>,
}

impl ReflectionData {
    /// List all the methods that made reflection calls.
    pub fn get_method_referenced(&self) -> HashSet<IdMethod> {
        self.invoke_data
            .iter()
            .map(|data| data.caller_method.clone())
            .chain(
                self.class_new_inst_data
                    .iter()
                    .map(|data| data.caller_method.clone())
                    .chain(
                        self.cnstr_new_inst_data
                            .iter()
                            .map(|data| data.caller_method.clone()),
                    ),
            )
            .collect()
    }

    /// List all data collected from called to `java.lang.reflect.Method.invoke()` made by
    /// `method`.
    pub fn get_invoke_data_for(
        &self,
        method: &IdMethod,
    ) -> HashMap<String, Vec<ReflectionInvokeData>> {
        let mut data = HashMap::new();
        for val in self
            .invoke_data
            .iter()
            .filter(|data| &data.caller_method == method)
        {
            let key = format!("THESEUS_ADDR_{:08X}", val.addr);
            let entry = data.entry(key).or_insert(vec![]);
            entry.push(val.clone());
        }
        data
    }
    /// List all data collected from called to `java.lang.Class.newInstance()` made by
    /// `method`.
    pub fn get_class_new_instance_data_for(
        &self,
        method: &IdMethod,
    ) -> HashMap<String, Vec<ReflectionClassNewInstData>> {
        let mut data = HashMap::new();
        for val in self
            .class_new_inst_data
            .iter()
            .filter(|data| &data.caller_method == method)
        {
            let key = format!("THESEUS_ADDR_{:08X}", val.addr);
            let entry = data.entry(key).or_insert(vec![]);
            entry.push(val.clone());
        }
        data
    }
    /// List all data collected from called to `java.lang.reflect.Constructor.newInstance()` made by
    /// `method`.
    pub fn get_cnstr_new_instance_data_for(
        &self,
        method: &IdMethod,
    ) -> HashMap<String, Vec<ReflectionCnstrNewInstData>> {
        let mut data = HashMap::new();
        for val in self
            .cnstr_new_inst_data
            .iter()
            .filter(|data| &data.caller_method == method)
        {
            let key = format!("THESEUS_ADDR_{:08X}", val.addr);
            let entry = data.entry(key).or_insert(vec![]);
            entry.push(val.clone());
        }
        data
    }
}

/// Structure storing the runtime information of a reflection call using
/// `java.lang.reflect.Method.invoke()`.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ReflectionInvokeData {
    /// The method called by `java.lang.reflect.Method.invoke()`
    pub method: IdMethod,
    /// The method calling `java.lang.reflect.Method.invoke()`
    pub caller_method: IdMethod,
    /// Address where the call to `java.lang.reflect.Method.invoke()` was made in `caller_method`.
    pub addr: usize,
    // TODO: variable number of args?
    // TODO: type of invoke?
}

/// Structure storing the runtime information of a reflection instanciation using
/// `java.lang.Class.newInstance()`.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ReflectionClassNewInstData {
    /// The constructor called by `java.lang.Class.newInstance()`
    pub constructor: IdMethod,
    /// The method calling `java.lang.Class.newInstance()`
    pub caller_method: IdMethod,
    /// Address where the call to `java.lang.Class.newInstance()` was made in `caller_method`.
    pub addr: usize,
}

/// Structure storing the runtime information of a reflection instanciation using
/// `java.lang.reflect.Constructor.newInstance()`.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ReflectionCnstrNewInstData {
    /// The constructor calleb by `java.lang.reflect.Constructor.newInstance()`
    pub constructor: IdMethod,
    /// The method calling `java.lang.reflect.Constructor.newInstance()`
    pub caller_method: IdMethod,
    /// Address where the call to `java.lang.Class.newInstance()` was made in `caller_method`.
    pub addr: usize,
}

pub struct RegistersInfo {
    pub array_index: u8,
    //pub array: u8,
    pub array_val: u8,
    pub array: u8,
    //pub original_array_index_reg: Option<u16>,
    //pub original_array_reg: Option<u16>,
    pub first_arg: u16,
    pub nb_arg_reg: u16,
}

impl RegistersInfo {
    const NB_U8_REG: u16 = 3;
    fn get_nb_added_reg(&self) -> u16 {
        3 + self.nb_arg_reg
    }
}

static MTH_INVOKE: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali(
    "Ljava/lang/reflect/Method;->invoke(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
)
.unwrap()
});
static MTH_GET_NAME: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getName()Ljava/lang/String;").unwrap()
});
static MTH_GET_PARAMS_TY: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getParameterTypes()[Ljava/lang/Class;")
        .unwrap()
});
static MTH_GET_RET_TY: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getReturnType()Ljava/lang/Class;").unwrap()
});
static MTH_GET_DEC_CLS: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Method;->getDeclaringClass()Ljava/lang/Class;")
        .unwrap()
});
static STR_EQ: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/String;->equals(Ljava/lang/Object;)Z").unwrap()
});
static CLASS_NEW_INST: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/Class;->newInstance()Ljava/lang/Object;").unwrap()
});
static CNSTR_NEW_INST: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali(
        "Ljava/lang/reflect/Constructor;->newInstance([Ljava/lang/Object;)Ljava/lang/Object;",
    )
    .unwrap()
});
static CNSTR_GET_PARAMS_TY: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Constructor;->getParameterTypes()[Ljava/lang/Class;")
        .unwrap()
});
static CNSTR_GET_DEC_CLS: LazyLock<IdMethod> = LazyLock::new(|| {
    IdMethod::from_smali("Ljava/lang/reflect/Constructor;->getDeclaringClass()Ljava/lang/Class;")
        .unwrap()
});

/// Function passed to [`androscalpel::Apk::load_apk`] to label the instructions of interest.
pub fn labeling(_mth: &IdMethod, ins: &Instruction, addr: usize) -> Option<String> {
    match ins {
        Instruction::InvokeVirtual { method, .. }
            if method == &*MTH_INVOKE
                || method == &*CLASS_NEW_INST
                || method == &*CNSTR_NEW_INST =>
        {
            Some(format!("THESEUS_ADDR_{addr:08X}"))
        }
        _ => None,
    }
}

// Interesting stuff: https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/verifier/reg_type.h;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=94
// https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/verifier/method_verifier.cc;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=5328
pub fn transform_method(meth: &mut Method, ref_data: &ReflectionData) -> Result<()> {
    // checking meth.annotations might be usefull at some point
    //println!("{}", meth.descriptor.__str__());
    let invoke_data = ref_data.get_invoke_data_for(&meth.descriptor);
    let class_new_inst_data = ref_data.get_class_new_instance_data_for(&meth.descriptor);
    let cnstr_new_inst_data = ref_data.get_cnstr_new_instance_data_for(&meth.descriptor);

    let code = meth
        .code
        .as_mut()
        .with_context(|| format!("Code not found in {}", meth.descriptor.__str__()))?;
    // TODO
    if code.registers_size + RegistersInfo::NB_U8_REG > u8::MAX as u16 {
        bail!("To many registers");
    }
    let mut register_info = RegistersInfo {
        array_index: code.registers_size as u8,
        array_val: (code.registers_size + 1) as u8,
        array: (code.registers_size + 2) as u8,
        //array: 0,
        first_arg: code.registers_size + 3,
        nb_arg_reg: 0,
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
                // TODO: recover from failure
                if method == &*MTH_INVOKE {
                    for ref_data in invoke_data.get(addr_label).unwrap_or(&vec![]) {
                        for ins in get_invoke_block(
                            ref_data,
                            args.as_slice(),
                            &mut register_info,
                            &end_label,
                            move_ret.clone(),
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
    code.insns = vec![];
    // Start the method by moving the parameter to their registers pre-transformation.
    let mut i = 0;
    for arg in &meth.descriptor.proto.get_parameters() {
        if arg.is_class() || arg.is_array() {
            code.insns.push(Instruction::MoveObject {
                from: code.registers_size - code.ins_size + i + register_info.get_nb_added_reg(),
                to: code.registers_size - code.ins_size + i,
            });
            i += 1;
        } else if arg.is_long() || arg.is_double() {
            code.insns.push(Instruction::MoveWide {
                from: code.registers_size - code.ins_size + i + register_info.get_nb_added_reg(),
                to: code.registers_size - code.ins_size + i,
            });
            i += 2;
        } else {
            code.insns.push(Instruction::Move {
                from: code.registers_size - code.ins_size + i + register_info.get_nb_added_reg(),
                to: code.registers_size - code.ins_size + i,
            });
            i += 1;
        }
    }
    if i != code.ins_size {
        warn!(
            "Method {} argument do not match code ins_size ({})",
            meth.descriptor.__str__(),
            code.ins_size
        );
    }
    // Add the new code
    code.insns.append(&mut new_insns);
    code.registers_size += register_info.get_nb_added_reg();

    Ok(())
}

/// Return the MoveResult{,Wide,Object} associated to the last instruction of the iterator.
/// TODO: return the list of pseudo instruction between the last instruction and the move result.
fn get_move_result<'a>(
    mut iter: impl Iterator<Item = &'a Instruction>,
) -> (Vec<Instruction>, Option<Instruction>) {
    if let Some(ins) = iter.next() {
        match ins {
            Instruction::MoveResult { .. }
            | Instruction::MoveResultWide { .. }
            | Instruction::MoveResultObject { .. } => return (vec![], Some(ins.clone())),
            _ => (), // break,
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
) -> Result<Vec<Instruction>> {
    let (method_obj, obj_inst, arg_arr) = if let &[a, b, c] = invoke_arg {
        (a, b, c)
    } else {
        bail!(
            "Method;->invoke arg should have exactrly 3 arguments, found {}",
            invoke_arg.len()
        );
    };
    if arg_arr > u8::MAX as u16 {
        // TODO
        bail!("Cannot transform invoke calls to a method using 16 bits register for its argument");
    }
    let nb_args = ref_data.method.proto.get_parameters().len();
    if reg_inf.nb_arg_reg < nb_args as u16 + 1 {
        reg_inf.nb_arg_reg = nb_args as u16 + 1;
    }

    let abort_label = format!(
        "end_static_call_to_{}_at_{}",
        ref_data.method.try_to_smali()?,
        "TODO_ADDR"
    );
    let mut insns = test_method(
        method_obj,
        ref_data.method.clone(),
        abort_label.clone(),
        reg_inf,
    );

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
    for (i, param) in ref_data.method.proto.get_parameters().iter().enumerate() {
        insns.push(Instruction::Const {
            reg: reg_inf.array_index,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: reg_inf.array_val,
            arr: arg_arr as u8, // TODO
            idx: reg_inf.array_index,
        });
        insns.push(Instruction::CheckCast {
            reg: reg_inf.array_val,
            lit: param.clone(),
        });
        insns.push(Instruction::MoveObject {
            from: reg_inf.array_val as u16,
            to: reg_inf.first_arg + 1 + i as u16,
        });
    }
    insns.push(Instruction::InvokeVirtual {
        method: ref_data.method.clone(),
        args: (reg_inf.first_arg..reg_inf.first_arg + 1 + nb_args as u16).collect(),
    });
    if let Some(move_result) = move_result {
        insns.push(move_result);
    }
    insns.push(Instruction::Goto {
        label: end_label.to_string(),
    });
    insns.push(Instruction::Label { name: abort_label });
    // We need a few u8 regs here. For now, we assumes we work with less than 256 reg.
    Ok(insns)
}

/// Generate bytecode that test if a `java.lang.reflect.Method` is equal to an [`IdMethod`]
///
/// - `method_obj_reg`: the register containing the `java.lang.reflect.Method`
/// - `id_method`: the expected [`IdMethod`].
/// - `abort_label`: the label where to jump if the method does not match `id_method`.
pub fn test_method(
    method_obj_reg: u16,
    id_method: IdMethod,
    abort_label: String,
    reg_inf: &mut RegistersInfo,
) -> Vec<Instruction> {
    // Check for arg type
    let mut insns = vec![
        Instruction::InvokeVirtual {
            method: MTH_GET_PARAMS_TY.clone(),
            args: vec![method_obj_reg],
        },
        Instruction::MoveResultObject { to: reg_inf.array },
        // First check  the number of args
        Instruction::ArrayLength {
            dest: reg_inf.array_index,
            arr: reg_inf.array,
        },
        Instruction::Const {
            reg: reg_inf.array_val,
            lit: id_method.proto.get_parameters().len() as i32,
        },
        Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        },
    ];
    // then the type of each arg
    for (i, param) in id_method.proto.get_parameters().into_iter().enumerate() {
        insns.push(Instruction::Const {
            reg: reg_inf.array_index,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: reg_inf.array_val,
            arr: reg_inf.array,
            idx: reg_inf.array_index,
        });
        insns.push(Instruction::ConstClass {
            reg: reg_inf.array_index, // wrong name, but available for tmp val
            lit: param,
        });
        insns.push(Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        })
    }
    insns.append(&mut vec![
        // Check the runtime method is the right one
        // Check Name
        Instruction::InvokeVirtual {
            method: MTH_GET_NAME.clone(),
            args: vec![method_obj_reg],
        },
        Instruction::MoveResultObject {
            to: reg_inf.array_index, // wrong name, but available for tmp val
        },
        Instruction::ConstString {
            reg: reg_inf.array_val, // wrong name, but available for tmp val
            lit: id_method.name.clone(),
        },
        Instruction::InvokeVirtual {
            method: STR_EQ.clone(),
            args: vec![reg_inf.array_index as u16, reg_inf.array_val as u16],
        },
        Instruction::MoveResult {
            to: reg_inf.array_index, // wrong name, but available for tmp val
        },
        Instruction::IfEqZ {
            a: reg_inf.array_index,
            label: abort_label.clone(),
        },
        // Check Return Type
        Instruction::InvokeVirtual {
            method: MTH_GET_RET_TY.clone(),
            args: vec![method_obj_reg],
        },
        Instruction::MoveResultObject {
            to: reg_inf.array_index, // wrong name, but available for tmp val
        },
        Instruction::ConstClass {
            reg: reg_inf.array_val, // wrong name, but available for tmp val
            lit: id_method.proto.get_return_type(),
        },
        Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        },
        // Check Declaring Type
        Instruction::InvokeVirtual {
            method: MTH_GET_DEC_CLS.clone(),
            args: vec![method_obj_reg],
        },
        Instruction::MoveResultObject {
            to: reg_inf.array_index, // wrong name, but available for tmp val
        },
        Instruction::ConstClass {
            reg: reg_inf.array_val, // wrong name, but available for tmp val
            lit: id_method.class_.clone(),
        },
        Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        },
    ]);
    insns
}

fn get_cnstr_new_inst_block(
    ref_data: &ReflectionCnstrNewInstData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
    end_label: &str,
    move_result: Option<Instruction>,
) -> Result<Vec<Instruction>> {
    let (cnst_reg, arg_arr) = if let &[a, b] = invoke_arg {
        (a, b)
    } else {
        bail!(
            "Method;->invoke arg should have exactrly 2 arguments, found {}",
            invoke_arg.len()
        );
    };
    if cnst_reg > u8::MAX as u16 {
        // TODO
        bail!("Cannot transform instantiation calls to a class stored in 16 bits register");
    }
    if reg_inf.first_arg > u8::MAX as u16 {
        // TODO
        bail!("Cannot transform instantiation calls to a class with first argument register greater than 255.");
    }
    //let cnst_reg = cnst_reg as u8;

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
    );
    for (i, param) in ref_data
        .constructor
        .proto
        .get_parameters()
        .iter()
        .enumerate()
    {
        insns.push(Instruction::Const {
            reg: reg_inf.array_index,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: reg_inf.array_val,
            arr: arg_arr as u8, // TODO
            idx: reg_inf.array_index,
        });
        insns.push(Instruction::CheckCast {
            reg: reg_inf.array_val,
            lit: param.clone(),
        });
        insns.push(Instruction::MoveObject {
            from: reg_inf.array_val as u16,
            to: reg_inf.first_arg + i as u16 + 1,
        });
    }
    insns.push(Instruction::NewInstance {
        reg: reg_inf.first_arg as u8,
        lit: ref_data.constructor.class_.clone(),
    });
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
pub fn test_cnstr(
    cnst_reg: u16,
    id_method: IdMethod,
    abort_label: String,
    reg_inf: &mut RegistersInfo,
) -> Vec<Instruction> {
    // Check for arg type
    let mut insns = vec![
        Instruction::InvokeVirtual {
            method: CNSTR_GET_PARAMS_TY.clone(),
            args: vec![cnst_reg],
        },
        Instruction::MoveResultObject { to: reg_inf.array },
        // First check  the number of args
        Instruction::ArrayLength {
            dest: reg_inf.array_index,
            arr: reg_inf.array,
        },
        Instruction::Const {
            reg: reg_inf.array_val,
            lit: id_method.proto.get_parameters().len() as i32,
        },
        Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        },
    ];
    // then the type of each arg
    for (i, param) in id_method.proto.get_parameters().into_iter().enumerate() {
        insns.push(Instruction::Const {
            reg: reg_inf.array_index,
            lit: i as i32,
        });
        insns.push(Instruction::AGetObject {
            dest: reg_inf.array_val,
            arr: reg_inf.array,
            idx: reg_inf.array_index,
        });
        insns.push(Instruction::ConstClass {
            reg: reg_inf.array_index, // wrong name, but available for tmp val
            lit: param,
        });
        insns.push(Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        })
    }
    insns.append(&mut vec![
        // Check Declaring Type
        Instruction::InvokeVirtual {
            method: CNSTR_GET_DEC_CLS.clone(),
            args: vec![cnst_reg],
        },
        Instruction::MoveResultObject {
            to: reg_inf.array_index, // wrong name, but available for tmp val
        },
        Instruction::ConstClass {
            reg: reg_inf.array_val, // wrong name, but available for tmp val
            lit: id_method.class_.clone(),
        },
        Instruction::IfNe {
            a: reg_inf.array_index,
            b: reg_inf.array_val,
            label: abort_label.clone(),
        },
    ]);
    insns
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
