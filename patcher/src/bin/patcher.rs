use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

use androscalpel::Apk;

use patcher::{
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
    #[arg(short, long)]
    zipalign: Option<PathBuf>,
    #[arg(short, long)]
    apksigner: Option<PathBuf>,
    #[arg(short, long)]
    path: PathBuf,
    #[arg(short, long)]
    reflection_data: PathBuf,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let mut apk = Apk::load_apk(File::open(&cli.path).unwrap(), labeling, false).unwrap();
    //println!("{:#?}", apk.list_classes());
    let mut json = String::new();
    File::open(&cli.reflection_data)
        .unwrap()
        .read_to_string(&mut json)
        .unwrap();
    let reflection_data: RuntimeData = serde_json::from_str(&json).unwrap();
    /*
    let reflection_data = RuntimeData {
        invoke_data: vec![
            ReflectionInvokeData {
                method: IdMethod::from_smali(
                    "Lcom/example/theseus/reflection/Reflectee;\
                    ->transfer\
                    (Ljava/lang/String;)Ljava/lang/String;",
                )
                .unwrap(),
                caller_method: IdMethod::from_smali(
                    "Lcom/example/theseus/reflection/MainActivity;\
                    ->callVirtualMethodReflectCall()V",
                )
                .unwrap(),
                addr: 0x2B,
            },
            ReflectionInvokeData {
                method: IdMethod::from_smali(
                    "Lcom/example/theseus/reflection/Reflectee;\
                    ->transfer(Ljava/lang/String;)Ljava/lang/String;",
                )
                .unwrap(),
                caller_method: IdMethod::from_smali(
                    "Lcom/example/theseus/reflection/MainActivity;\
                    ->callConstructorVirtualMethodReflectConstr()V",
                )
                .unwrap(),
                addr: 0x38,
            },
            ReflectionInvokeData {
                method: IdMethod::from_smali(
                    "Lcom/example/theseus/reflection/Reflectee;\
                    ->transfer(Ljava/lang/String;)Ljava/lang/String;",
                )
                .unwrap(),
                caller_method: IdMethod::from_smali(
                    "Lcom/example/theseus/reflection/MainActivity;\
                    ->callVirtualMethodReflectOldConst()V",
                )
                .unwrap(),
                addr: 0x28,
            },
        ],
        class_new_inst_data: vec![ReflectionClassNewInstData {
            constructor: IdMethod::from_smali(
                "Lcom/example/theseus/reflection/Reflectee;\
                -><init>()V",
            )
            .unwrap(),
            caller_method: IdMethod::from_smali(
                "Lcom/example/theseus/reflection/MainActivity;\
                ->callVirtualMethodReflectOldConst()V",
            )
            .unwrap(),
            addr: 0x12,
        }],
        cnstr_new_inst_data: vec![ReflectionCnstrNewInstData {
            constructor: IdMethod::from_smali(
                "Lcom/example/theseus/reflection/Reflectee;\
                -><init>(Ljava/lang/String;)V",
            )
            .unwrap(),
            caller_method: IdMethod::from_smali(
                "Lcom/example/theseus/reflection/MainActivity;\
                ->callConstructorVirtualMethodReflectConstr()V",
            )
            .unwrap(),
            addr: 0x22,
        }],
    };
    println!("{}", serde_json::to_string(&reflection_data).unwrap());
    */
    for method in reflection_data.get_method_referenced().iter() {
        if let Some(class) = apk.get_class_mut(&method.class_) {
            //println!("{:#?}", class.direct_methods.keys());
            //println!("{:#?}", class.virtual_methods.keys());
            let method = class.virtual_methods.get_mut(method).unwrap();
            transform_method(method, &reflection_data).unwrap();
        }
    }
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
    // TODO: aapt would be a lot more stable
    apk_frauder::replace_dex(
        cli.path,
        cli.out,
        &mut dex_files,
        cli.keystore,
        cli.zipalign,
        cli.apksigner,
        None::<HashMap<_, Option<Cursor<&[u8]>>>>,
    );
}
