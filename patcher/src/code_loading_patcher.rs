use std::collections::HashMap;
use std::fs::File;

use androscalpel::{Apk, IdType};
use anyhow::Result;
use clap::ValueEnum;

use crate::runtime_data::RuntimeData;

#[derive(ValueEnum, Debug, PartialEq, Clone, Copy, Default)]
pub enum CodePatchingStrategy {
    #[default]
    Naive,
    ModelClassLoaders,
}

pub fn insert_code(
    strategy: CodePatchingStrategy,
    apk: &mut Apk,
    data: &RuntimeData,
) -> Result<()> {
    match strategy {
        CodePatchingStrategy::Naive => insert_code_naive(apk, data),
        CodePatchingStrategy::ModelClassLoaders => insert_code_model_class_loaders(apk, data),
    }
}

/// Insert statically bytecode that was loaded from other source at runtime.
/// For now, we ignore class collision.
fn insert_code_naive(apk: &mut Apk, data: &RuntimeData) -> Result<()> {
    for dyn_data in &data.dyn_code_load {
        for file in &dyn_data.files {
            let file = File::open(file)?;
            apk.add_code(file, crate::labeling, false)?;
        }
    }
    Ok(())
}

fn insert_code_model_class_loaders(apk: &mut Apk, data: &RuntimeData) -> Result<()> {
    let mut class_loaders = HashMap::new();
    class_loaders.insert(
        "MAIN".to_string(),
        ClassLoader {
            parent: None,
            class: IdType::from_smali("Ljava/lang/Boolean;").unwrap(),
            apk: ApkOrRef::Ref(apk),
        },
    );
    for dyn_data in &data.dyn_code_load {
        let mut apk = Apk::new();
        let class = dyn_data.classloader_class.clone();
        for file in &dyn_data.files {
            let file = File::open(file)?;
            apk.add_code(file, crate::labeling, false)?;
        }
        assert!(!class_loaders.contains_key(&dyn_data.classloader));
        class_loaders.insert(
            dyn_data.classloader.clone(),
            ClassLoader {
                parent: None,
                class,
                apk: ApkOrRef::Owned(apk),
            },
        );
    }
    // TODO: list colliding classes
    // TODO: rename colliding classes according to class laoder
    // TODO: get the ClassLoader::parent values...
    // TODO: model the delegation behavior and rename ref to class accordingly
    // TODO: update Runtime Data to reflect the name change
    todo!()
}

/// Structure modelizing a class loader.
#[derive(Debug, PartialEq)]
struct ClassLoader<'a> {
    pub parent: Option<String>,
    pub class: IdType,
    pub apk: ApkOrRef<'a>,
}

impl ClassLoader<'_> {
    pub fn _apk(&mut self) -> &mut Apk {
        match &mut self.apk {
            ApkOrRef::Owned(ref mut apk) => apk,
            ApkOrRef::Ref(ref mut apk) => apk,
        }
    }
}

#[derive(Debug, PartialEq)]
enum ApkOrRef<'a> {
    Owned(Apk),
    Ref(&'a mut Apk),
}
