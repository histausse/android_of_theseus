use androscalpel::{IdMethod, Instruction, Method};
use anyhow::{bail, Context, Result};

pub mod get_apk;

// TODO:
// Check what
// https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/reflection.cc;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=698
// does.

/// Structure storing the runtime information of a reflection call.
pub struct ReflectionData {
    pub method: IdMethod,
    // TODO: variable number of args?
    // TODO: type of invoke?
}

struct RegistersInfo {
    pub array_index: u8,
    //pub array: u8,
    pub array_val: u8,
    //pub original_array_index_reg: Option<u16>,
    //pub original_array_reg: Option<u16>,
    pub first_arg: u16,
    pub nb_arg_reg: u16,
}

impl RegistersInfo {
    const NB_U8_REG: u16 = 2;
    fn get_nb_added_reg(&self) -> u16 {
        2 + self.nb_arg_reg
    }
}

const INVOKE: &str =
    "Ljava/lang/reflect/Method;->invoke(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;";

// Interesting stuff: https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/verifier/reg_type.h;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=94
// https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/verifier/method_verifier.cc;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=5328
pub fn transform_method(meth: &mut Method, ref_data: &ReflectionData) -> Result<()> {
    let invoke = IdMethod::from_smali(INVOKE)?;
    // checking meth.annotations might be usefull at some point
    let code = meth
        .code
        .as_mut()
        .with_context(|| format!("Code not found in {}", meth.descriptor.__str__()))?;
    println!(
        "registers_size: {}\nins_size: {}\nouts_size: {}",
        code.registers_size, code.ins_size, code.outs_size,
    );
    // TODO
    if code.registers_size + RegistersInfo::NB_U8_REG > u8::MAX as u16 {
        bail!("To many registers");
    }
    let mut register_info = RegistersInfo {
        array_index: code.registers_size as u8,
        array_val: (code.registers_size + 1) as u8,
        //array: 0,
        first_arg: code.registers_size + 2,
        nb_arg_reg: 0,
    };
    let mut new_insns = vec![];
    for ins in &code.insns {
        match ins {
            Instruction::InvokeVirtual { method, args } if method == &invoke => {
                // TODO move ret ?
                // TODO: rever from get_invoke_block failure
                for ins in
                    get_invoke_block(ref_data, args.as_slice(), &mut register_info)?.into_iter()
                {
                    println!("  \x1b[92m{}\x1b[0m", ins.__str__());
                    new_insns.push(ins);
                }
                //new_insns.push(ins.clone());
                println!("  \x1b[91m{}\x1b[0m", ins.__str__());
            }
            ins => {
                println!("  {}", ins.__str__());
                new_insns.push(ins.clone());
            }
        }
    }
    // TODO: scalar type
    code.insns = vec![];
    // Start the method by moving the parameter to their registers pre-transformation.
    for i in 0..code.ins_size {
        code.insns.push(Instruction::MoveObject {
            from: code.registers_size - code.ins_size + i + register_info.get_nb_added_reg(),
            to: code.registers_size - code.ins_size + i,
        });
    }
    // Add the new code
    code.insns.append(&mut new_insns);
    code.registers_size += register_info.get_nb_added_reg();

    println!("\nnew code:\n");
    for ins in &code.insns {
        println!("  {}", ins.__str__());
    }
    Ok(())
}

fn get_invoke_block(
    ref_data: &ReflectionData,
    invoke_arg: &[u16],
    reg_inf: &mut RegistersInfo,
) -> Result<Vec<Instruction>> {
    let (_method_obj, obj_inst, arg_arr) = if let &[a, b, c] = invoke_arg {
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
    let mut insns = vec![];
    insns.push(Instruction::MoveObject {
        from: obj_inst,
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
    // We need a few u8 regs here. For now, we assumes we work with less than 256 reg.
    Ok(insns)
}
