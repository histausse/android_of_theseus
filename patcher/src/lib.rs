use androscalpel::{IdMethod, Instruction};

pub mod dex_types;
pub mod reflection_patcher;
pub mod register_manipulation;
pub mod runtime_data;
use dex_types::*;

// TODO:
// Check what
// https://cs.android.com/android/platform/superproject/main/+/main:art/runtime/reflection.cc;drc=83db0626fad8c6e0508754fffcbbd58e539d14a5;l=698
// does.

/// Inject arbitrary text in the instructions array as 'source file' debug info.
/// It's cursed, but it work XD
fn _debug_info(data: &str) -> Vec<Instruction> {
    data.split("\n")
        .map(|data| Instruction::DebugSourceFile {
            file: Some(format!("  {data: <70}").into()),
        })
        .collect()
}

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
