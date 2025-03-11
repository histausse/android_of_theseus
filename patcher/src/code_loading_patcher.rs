use std::fs::File;

use androscalpel::Apk;
use anyhow::Result;

use crate::runtime_data::RuntimeData;

/// Insert statically bytecode that was loaded from other source at runtime.
/// For now, we ignore class collision.
pub fn insert_code(apk: &mut Apk, data: &RuntimeData) -> Result<()> {
    for dyn_data in &data.dyn_code_load {
        for file in &dyn_data.files {
            let file = File::open(file)?;
            apk.add_code(file, crate::labeling, false)?;
        }
    }
    Ok(())
}
