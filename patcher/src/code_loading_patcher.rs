use std::collections::{HashMap, HashSet};
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
    let mut class_defined = apk.list_classes();
    let mut class_loaders = HashMap::new();
    class_loaders.insert(
        "MAIN".to_string(),
        ClassLoader {
            id: "MAIN".to_string(),
            parent: None,
            class: IdType::from_smali("Ljava/lang/Boolean;").unwrap(),
            apk: ApkOrRef::Ref(apk),
            renamed_classes: HashSet::new(),
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

        let classes = apk.list_classes();
        let mut class_loader = ClassLoader {
            id: dyn_data.classloader.clone(),
            parent: None,
            class,
            apk: ApkOrRef::Owned(apk),
            renamed_classes: HashSet::new(),
        };
        let collisions = class_defined.intersection(&classes);
        for cls in collisions {
            class_loader.rename_classdef(cls);
        }
        class_defined.extend(classes);

        class_loaders.insert(dyn_data.classloader.clone(), class_loader);
    }
    // TODO: rename colliding classes according to class laoder
    // TODO: get the ClassLoader::parent values...
    // TODO: model the delegation behavior and rename ref to class accordingly
    // TODO: update Runtime Data to reflect the name change
    todo!()
}

/// Structure modelizing a class loader.
#[derive(Debug, PartialEq)]
struct ClassLoader<'a> {
    pub id: String,
    pub parent: Option<String>,
    pub class: IdType,
    pub apk: ApkOrRef<'a>,
    pub renamed_classes: HashSet<IdType>,
}

impl ClassLoader<'_> {
    pub fn _apk(&mut self) -> &mut Apk {
        match &mut self.apk {
            ApkOrRef::Owned(ref mut apk) => apk,
            ApkOrRef::Ref(ref mut apk) => apk,
        }
    }

    pub fn rename_classdef(&mut self, cls: &IdType) {
        use androscalpel::SmaliName;
        println!(
            "TODO: rename {} -> {}_{}",
            cls.try_to_smali().unwrap(),
            cls.try_to_smali().unwrap(),
            self.id
        );
    }
}

#[derive(Debug, PartialEq)]
enum ApkOrRef<'a> {
    Owned(Apk),
    Ref(&'a mut Apk),
}
