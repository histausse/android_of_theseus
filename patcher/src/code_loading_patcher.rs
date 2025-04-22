use std::collections::{HashMap, HashSet};
use std::fs::File;

use androscalpel::{Apk, DexString, IdType, VisitorMut};
use anyhow::{Context, Result};
use clap::ValueEnum;

use crate::dex_types::DELEGATE_LAST_CLASS_LOADER;
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
    data: &mut RuntimeData,
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
        for file_name in &dyn_data.files {
            let file = File::open(file_name)?;
            apk.add_code(file, crate::labeling, false)
                .with_context(|| {
                    format!(
                        "Could not add code from {}",
                        file_name.to_str().unwrap_or("<failed to parse file name>")
                    )
                })?;
        }
    }
    Ok(())
}

fn insert_code_model_class_loaders(apk: &mut Apk, runtime_data: &mut RuntimeData) -> Result<()> {
    let mut class_defined = apk.list_classes();
    let mut class_redefined = HashSet::new();
    let mut class_loaders = HashMap::new();
    let main_cl_id = runtime_data
        .apk_cl_id
        .clone()
        .unwrap_or_else(|| "MAIN".to_string());
    class_loaders.insert(
        main_cl_id.clone(),
        ClassLoader {
            id: main_cl_id.clone(),
            parent: None,
            class: IdType::from_smali("Ljava/lang/Boolean;").unwrap(),
            apk: ApkOrRef::Ref(apk),
            renamed_classes: HashMap::new(),
        },
    );
    // -- Rename class def --
    for dyn_data in &runtime_data.dyn_code_load {
        let mut apk = Apk::new();
        let class = dyn_data.classloader_class.clone();
        for file_name in &dyn_data.files {
            let file = File::open(file_name)?;
            apk.add_code(file, crate::labeling, false)
                .with_context(|| {
                    format!(
                        "Could not add code from {}",
                        file_name.to_str().unwrap_or("<failed to parse file name>")
                    )
                })?;
        }

        assert!(
            !class_loaders.contains_key(&dyn_data.classloader),
            "The same class loader should not appear twice in runtime_data.dyn_code_load"
        );

        let classes = apk.list_classes();
        let mut class_loader = ClassLoader {
            id: dyn_data.classloader.clone(),
            parent: dyn_data.classloader_parent.clone(),
            class,
            apk: ApkOrRef::Owned(apk),
            renamed_classes: HashMap::new(),
        };
        let collisions = class_defined.intersection(&classes);
        for cls in collisions {
            class_loader.rename_classdef(cls)?;
            class_redefined.insert(cls.clone());
        }
        class_defined.extend(classes);

        class_loaders.insert(dyn_data.classloader.clone(), class_loader);
    }

    // -- Rename class ref --
    let mut renamers: HashMap<_, _> = class_loaders
        .values()
        .map(|cl| {
            (
                cl.id.clone(),
                RenameTypeVisitor {
                    new_names: cl.get_ref_new_names(&class_redefined, &class_loaders),
                },
            )
        })
        .collect();
    class_loaders = class_loaders
        .into_iter()
        .map(|(k, v)| {
            // main_cl_id is not owned, so we can't use the visitor on it,
            // and anyway, there is no reason to rename ref in it (its only parent
            // sould be the classbootloader, and we don't handle changing class loader
            // on the fly)
            if let Some(renamer) = renamers.get_mut(&k) {
                if k != main_cl_id {
                    v.rename_refs(renamer).map(|v| (k, v))
                } else {
                    Ok((k, v))
                }
            } else {
                Ok((k, v))
            }
        })
        .collect::<Result<_>>()?;

    // -- update Runtime Data to reflect the name change --
    runtime_data.invoke_data.iter_mut().for_each(|data| {
        if let Some(visitor) = renamers.get_mut(&data.method_cl_id) {
            match visitor.visit_method_id(data.method.clone()) {
                Err(err) => log::warn!(
                    "Failed to generate new name for {} from {}: {err}",
                    data.method.__str__(),
                    data.method_cl_id
                ),
                Ok(new_method) => data.renamed_method = Some(new_method),
            }
        }
        if let Some(visitor) = renamers.get_mut(&data.caller_cl_id) {
            match visitor.visit_method_id(data.caller_method.clone()) {
                Err(err) => log::warn!(
                    "Failed to generate new name for {} from {}: {err}",
                    data.caller_method.__str__(),
                    data.caller_cl_id
                ),
                Ok(new_method) => data.renamed_caller_method = Some(new_method),
            }
        }
    });
    runtime_data
        .class_new_inst_data
        .iter_mut()
        .for_each(|data| {
            if let Some(visitor) = renamers.get_mut(&data.constructor_cl_id) {
                match visitor.visit_method_id(data.constructor.clone()) {
                    Err(err) => log::warn!(
                        "Failed to generate new name for {} from {}: {err}",
                        data.constructor.__str__(),
                        data.constructor_cl_id
                    ),
                    Ok(new_method) => data.renamed_constructor = Some(new_method),
                }
            }
            if let Some(visitor) = renamers.get_mut(&data.caller_cl_id) {
                match visitor.visit_method_id(data.caller_method.clone()) {
                    Err(err) => log::warn!(
                        "Failed to generate new name for {} from {}: {err}",
                        data.caller_method.__str__(),
                        data.caller_cl_id
                    ),
                    Ok(new_method) => data.renamed_caller_method = Some(new_method),
                }
            }
        });
    runtime_data
        .cnstr_new_inst_data
        .iter_mut()
        .for_each(|data| {
            if let Some(visitor) = renamers.get_mut(&data.constructor_cl_id) {
                match visitor.visit_method_id(data.constructor.clone()) {
                    Err(err) => log::warn!(
                        "Failed to generate new name for {} from {}: {err}",
                        data.constructor.__str__(),
                        data.constructor_cl_id
                    ),
                    Ok(new_method) => data.renamed_constructor = Some(new_method),
                }
            }
            if let Some(visitor) = renamers.get_mut(&data.caller_cl_id) {
                match visitor.visit_method_id(data.caller_method.clone()) {
                    Err(err) => log::warn!(
                        "Failed to generate new name for {} from {}: {err}",
                        data.caller_method.__str__(),
                        data.caller_cl_id
                    ),
                    Ok(new_method) => data.renamed_caller_method = Some(new_method),
                }
            }
        });

    // -- inject code to apk --
    let apk = match class_loaders.remove(&main_cl_id).unwrap().apk {
        ApkOrRef::Ref(apk) => apk,
        _ => {
            panic!("Main APK is not stored as ref?")
        }
    };
    for (_, ClassLoader { apk: other, .. }) in class_loaders.into_iter() {
        match other {
            ApkOrRef::Owned(other) => {
                apk.merge(other);
            }
            _ => {
                panic!("Secondary APK is not stored as owned?")
            }
        }
    }
    Ok(())
}

/// Structure modelizing a class loader.
#[derive(Debug, PartialEq)]
struct ClassLoader<'a> {
    pub id: String,
    pub parent: Option<String>,
    pub class: IdType,
    pub apk: ApkOrRef<'a>,
    pub renamed_classes: HashMap<IdType, IdType>,
}

impl ClassLoader<'_> {
    pub fn apk_mut(&mut self) -> &mut Apk {
        match &mut self.apk {
            ApkOrRef::Owned(apk) => apk,
            ApkOrRef::Ref(apk) => apk,
        }
    }
    pub fn apk(&self) -> &Apk {
        match &self.apk {
            ApkOrRef::Owned(apk) => apk,
            ApkOrRef::Ref(apk) => apk,
        }
    }

    /// Rename the definition of `cls` in the class loader.
    ///
    pub fn rename_classdef(&mut self, cls: &IdType) -> Result<()> {
        let id = self.id.clone();
        let mut i = 0;
        let name = if let Some(name) = cls.get_class_name() {
            name
        } else {
            log::warn!("Tried to rename non class type {}", cls.__str__());
            return Ok(());
        };
        let new_name = loop {
            let prefix: DexString = if i == 0 {
                format!("theseus-dedup/{}/", self.id).into()
            } else {
                format!("theseus-dedup/{}-{i}/", self.id).into()
            };
            let new_name = IdType::class_from_dex_string(&prefix.concatenate(&name));
            if self.apk().get_class(&new_name).is_none() {
                break new_name;
            }
            i += 1;
        };

        let class = self.apk_mut().remove_class(cls, None)?.with_context(|| {
            format!(
                "Try to rename classdef of {} in class loader {}, but classdef not found",
                cls.__str__(),
                &id
            )
        })?;
        let class = RenameTypeVisitor {
            new_names: [(cls.clone(), new_name.clone())].into(),
        }
        .visit_class(class)?;
        self.apk_mut().add_class("classes.dex", class)?;

        self.renamed_classes.insert(cls.clone(), new_name);
        Ok(())
    }

    pub fn get_ref_new_names(
        &self,
        tys: &HashSet<IdType>,
        class_loaders: &HashMap<String, Self>,
    ) -> HashMap<IdType, IdType> {
        let r: HashMap<IdType, IdType> = tys
            .iter()
            .map(|ty| {
                (
                    ty.clone(),
                    self.get_ref_new_name(ty, class_loaders)
                        .unwrap_or(ty.clone()),
                )
            })
            .collect();
        println!("rename for {}", self.id);
        for (old, new) in &r {
            println!("    {} -> {}", old.__str__(), new.__str__());
        }
        r
    }

    /// Return the new name of a type after class renaming.
    /// This method select the right renamed type by modeling the behavior of
    /// the android SDK class loaders.
    /// If the class loader is not a class loader from the android SDK, default
    /// to the behavior of DexClassLoader: Platform classes have precedence over
    /// classes defined by a parent classloader that have precedence over classes
    /// defined by the classloader itself.
    pub fn get_ref_new_name(
        &self,
        ty: &IdType,
        class_loaders: &HashMap<String, Self>,
    ) -> Option<IdType> {
        if ty.is_platform_class() {
            // Platform classes have precedence for all android SDK classloader.
            return Some(ty);
        }
        if self.class == *DELEGATE_LAST_CLASS_LOADER {
            if let Some(new_ty) = self.renamed_classes.get(ty) {
                return Some(new_ty.clone());
            } else if self.apk().get_class(ty).is_some() {
                return Some(ty.clone());
            }
        }
        if let Some(ref parent_id) = self.parent {
            if let Some(parent) = class_loaders.get(parent_id) {
                if let Some(new_ty) = parent.get_ref_new_name(ty, class_loaders) {
                    return Some(new_ty);
                }
            } else {
                log::warn!("Class Loader {}({}) has parent {}, but parent was not found in class loader list", self.id, self.class.__str__(), parent_id);
            }
        }
        if self.class == *DELEGATE_LAST_CLASS_LOADER {
            return None;
        }
        if let Some(new_ty) = self.renamed_classes.get(ty) {
            Some(new_ty.clone())
        } else if self.apk().get_class(ty).is_some() {
            Some(ty.clone())
        } else {
            None
        }
    }

    pub fn rename_refs(self, renamer: &mut RenameTypeVisitor) -> Result<Self> {
        Ok(Self {
            apk: match self.apk {
                ApkOrRef::Owned(apk) => match renamer.visit_apk(apk) {
                    Err(err) => {
                        log::error!(
                            "Failed to rename refs in apk of {}({})): {err}",
                            self.id,
                            self.class.__str__()
                        );
                        return Err(err);
                    }
                    Ok(apk) => ApkOrRef::Owned(apk),
                },
                ApkOrRef::Ref(apk) => ApkOrRef::Ref(apk),
            },
            ..self
        })
    }
}

struct RenameTypeVisitor {
    pub new_names: HashMap<IdType, IdType>,
}

impl VisitorMut for RenameTypeVisitor {
    fn visit_type(&mut self, id: IdType) -> Result<IdType> {
        match self.new_names.get(&id) {
            Some(newid) => Ok(newid.clone()),
            None => Ok(id.clone()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum ApkOrRef<'a> {
    Owned(Apk),
    Ref(&'a mut Apk),
}
