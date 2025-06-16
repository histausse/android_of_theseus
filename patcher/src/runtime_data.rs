use androscalpel::{IdMethod, IdType};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct RuntimeData {
    pub invoke_data: Vec<ReflectionInvokeData>,
    pub class_new_inst_data: Vec<ReflectionClassNewInstData>,
    pub cnstr_new_inst_data: Vec<ReflectionCnstrNewInstData>,
    pub dyn_code_load: Vec<DynamicCodeLoadingData>,
    /// The id of the class loader of the apk (the main classloader)
    pub apk_cl_id: Option<String>,
    /// Additionnal classloader data.
    pub classloaders: HashMap<String, ClassLoaderData>,
    /// Additionnal application data.
    pub app_info: Option<AppInfo>,
}

impl RuntimeData {
    pub fn dedup(&mut self) {
        self.invoke_data.sort();
        self.invoke_data.dedup();
        self.class_new_inst_data.sort();
        self.class_new_inst_data.dedup();
        self.cnstr_new_inst_data.sort();
        self.cnstr_new_inst_data.dedup();
        // TODO; dedup dyn_code_load?
    }
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
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, PartialOrd, Ord)]
pub struct ReflectionInvokeData {
    /// The method called by `java.lang.reflect.Method.invoke()` (at runtime)
    pub method: IdMethod,
    /// The id of the classloader defining the constructor
    pub method_cl_id: String,
    /// The name of the method to call statically.
    pub renamed_method: Option<IdMethod>,
    /// The method calling `java.lang.reflect.Method.invoke()` (at runtime)
    pub caller_method: IdMethod,
    /// The id of the classloader defining the caller method
    pub caller_cl_id: String,
    /// The name of the method that call the method (statically)
    pub renamed_caller_method: Option<IdMethod>,
    /// Address where the call to `java.lang.reflect.Method.invoke()` was made in `caller_method`.
    pub addr: usize,
    /// If the method is static (static method don't take 'this' as argument)
    pub is_static: bool,
}

impl ReflectionInvokeData {
    pub fn get_static_callee(&self) -> IdMethod {
        self.renamed_method
            .clone()
            .unwrap_or_else(|| self.method.clone())
    }
}

/// Structure storing the runtime information of a reflection instanciation using
/// `java.lang.Class.newInstance()`.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, PartialOrd, Ord)]
pub struct ReflectionClassNewInstData {
    /// The constructor called by `java.lang.Class.newInstance()`
    pub constructor: IdMethod,
    /// The id of the classloader defining the constructor
    pub constructor_cl_id: String,
    /// The name of the constructor to call statically.
    pub renamed_constructor: Option<IdMethod>,
    /// The method calling `java.lang.Class.newInstance()`
    pub caller_method: IdMethod,
    /// The id of the classloader defining the caller method
    pub caller_cl_id: String,
    /// The name of the method that call the method (statically)
    pub renamed_caller_method: Option<IdMethod>,
    /// Address where the call to `java.lang.Class.newInstance()` was made in `caller_method`.
    pub addr: usize,
}

impl ReflectionClassNewInstData {
    pub fn get_static_constructor(&self) -> IdMethod {
        self.renamed_constructor
            .clone()
            .unwrap_or_else(|| self.constructor.clone())
    }
}

/// Structure storing the runtime information of a reflection instanciation using
/// `java.lang.reflect.Constructor.newInstance()`.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, PartialOrd, Ord)]
pub struct ReflectionCnstrNewInstData {
    /// The constructor calleb by `java.lang.reflect.Constructor.newInstance()`
    pub constructor: IdMethod,
    /// The id of the classloader defining the constructor
    pub constructor_cl_id: String,
    /// The name of the constructor to call statically.
    pub renamed_constructor: Option<IdMethod>,
    /// The method calling `java.lang.reflect.Constructor.newInstance()`
    pub caller_method: IdMethod,
    /// The id of the classloader defining the caller method
    pub caller_cl_id: String,
    /// The name of the method that call the method (statically)
    pub renamed_caller_method: Option<IdMethod>,
    /// Address where the call to `java.lang.Class.newInstance()` was made in `caller_method`.
    pub addr: usize,
}

impl std::fmt::Display for ReflectionCnstrNewInstData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "RefCnstr {{ constructor: {} ({}), renamed: {}, caller: {} ({}), renamed_caller: {}, addr: 0x{:x} }}",
            self.constructor.__str__(),
            self.constructor_cl_id,
            self.renamed_constructor
                .as_ref()
                .map(|id| id.__str__())
                .as_deref()
                .unwrap_or("None"),
                self.caller_method.__str__(),self.caller_cl_id, self.renamed_caller_method.as_ref()
                .map(|id| id.__str__())
                .as_deref()
                .unwrap_or("None"), self.addr
        )
    }
}

impl ReflectionCnstrNewInstData {
    pub fn get_static_constructor(&self) -> IdMethod {
        self.renamed_constructor
            .clone()
            .unwrap_or_else(|| self.constructor.clone())
    }
}

/// Structure storing the runtime information of a dynamic code loading.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct DynamicCodeLoadingData {
    /// The type of the class loader used to load the code.
    pub classloader_class: IdType,
    /// An identifier for the classloader, valid for one specific run of the application.
    pub classloader: String,
    /// An identifier for the parent classloader, valid for one specific run of the applications.
    pub classloader_parent: Option<String>,
    /// The path to the files storing the .dex/.apk/other bytecode loaded.
    pub files: Vec<PathBuf>,
}

/// Structure storing the runtime information of a classloader.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ClassLoaderData {
    /// Id of the classloader. This value is unique for *one* run of the apk.
    pub id: String,
    /// The Id of the parent classloader if it exists.
    pub parent_id: Option<String>,
    /// The string representation of the classloader. Not verry relayable but our best option to
    /// distinguish classloader at runtime.
    #[serde(rename = "str")]
    pub string_representation: String,
    /// The class of the class loader.
    pub cname: IdType,
}

/// Structure storing application information
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct AppInfo {
    pub data_dir: String,
    pub device_protected_data_dir: String,
    pub native_library_dir: String,
    pub public_source_dir: String,
    //pub shared_library_files: Option<Vec<String>>,
    pub source_dir: String,
    //pub split_names: Option<Vec<String>>,
    pub split_public_source_dirs: Option<Vec<String>>,
    pub split_source_dirs: Option<String>,
    pub actual_source_dir: String,
}
