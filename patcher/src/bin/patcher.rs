use anyhow::Context;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use androscalpel::{Apk, Class, IdType};

use androscalpel::SmaliName;
use patcher::{
    code_loading_patcher::{CodePatchingStrategy, insert_code},
    labeling,
    reflection_patcher::transform_method,
    runtime_data::RuntimeData, // ReflectionInvokeData, ReflectionClassNewInstData, ReflectionCnstrNewInstData,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[arg(short, long)]
    out: PathBuf,
    #[arg(short, long)]
    keystore: PathBuf,
    #[arg(long)]
    keypassword: Option<String>,
    #[arg(short, long)]
    zipalign: Option<PathBuf>,
    #[arg(short, long)]
    apksigner: Option<PathBuf>,
    #[arg(short, long)]
    path: PathBuf,
    #[arg(short, long)]
    runtime_data: PathBuf,
    #[arg(short, long, default_value_t, value_enum)]
    code_loading_patch_strategy: CodePatchingStrategy,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let mut apk = Apk::load_apk(File::open(&cli.path).unwrap(), labeling, false).unwrap();
    //println!("{:#?}", apk.list_classes());
    let mut json = String::new();
    File::open(&cli.runtime_data)
        .unwrap()
        .read_to_string(&mut json)
        .unwrap();
    let mut rt_data: RuntimeData = serde_json::from_str(&json).unwrap();

    // Dynamic Loading
    insert_code(cli.code_loading_patch_strategy, &mut apk, &mut rt_data).unwrap();
    let rt_data = rt_data; // not mut anymore

    // Reflection
    let mut test_methods = HashMap::new();
    let test_class = IdType::class("theseus/T");
    for method in rt_data.get_method_referenced().iter() {
        if let Some(class) = apk.get_class_mut(&method.class_) {
            //println!("{:#?}", class.direct_methods.keys());
            //println!("{:#?}", class.virtual_methods.keys());
            let method = if let Some(method) = class.virtual_methods.get_mut(method) {
                method
            } else {
                class
                    .direct_methods
                    .get_mut(method)
                    .with_context(|| {
                        format!(
                            "method {} not found in {}",
                            method.try_to_smali().unwrap(),
                            class.descriptor.try_to_smali().unwrap()
                        )
                    })
                    .unwrap()
            };
            transform_method(method, &rt_data, test_class.clone(), &mut test_methods).unwrap();
        }
    }
    let mut class = Class::new(test_class.get_name()).unwrap();
    class.is_final = true;
    class.direct_methods = test_methods
        .into_values()
        .map(|v| (v.descriptor.clone(), v))
        .collect();
    apk.add_class("classes.dex", class).unwrap();
    let mut dex_files = vec![];
    let mut files = apk.gen_raw_dex().unwrap();
    let mut i = 0;
    loop {
        let name = if i == 0 {
            "classes.dex".into()
        } else {
            format!("classes{}.dex", i + 1)
        };
        if let Some(file) = files.remove(&name) {
            dex_files.push(Cursor::new(file))
        } else {
            break;
        }
        i += 1;
    }
    // TODO: aapt would be a lot more stable?
    apk_frauder::replace_dex(
        cli.path,
        cli.out,
        &mut dex_files,
        cli.keystore,
        cli.zipalign,
        cli.apksigner,
        cli.keypassword.as_deref(),
        None::<HashMap<_, Option<Cursor<&[u8]>>>>,
    )
    .unwrap();
}
