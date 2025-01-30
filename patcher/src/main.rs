use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;

use androscalpel::{IdMethod, IdType};

use patcher::get_apk::{get_apk, ApkLocation};
use patcher::{transform_method, ReflectionData};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[clap(flatten)]
    apk: ApkLocation,
    #[arg(short, long)]
    out: PathBuf,
    #[arg(short, long)]
    keystore: PathBuf,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let mut apk = get_apk(&cli.apk);
    //println!("{:#?}", apk.list_classes());
    let class = apk
        .get_class_mut(
            &IdType::new("Lcom/example/theseus/reflection/MainActivity;".into()).unwrap(),
        )
        .unwrap();
    //println!("{:#?}", class.direct_methods.keys());
    //println!("{:#?}", class.virtual_methods.keys());
    let method = class
        .virtual_methods
        .get_mut(
            &IdMethod::from_smali(
                "Lcom/example/theseus/reflection/MainActivity;->callVirtualMethodReflectCall()V",
            )
            .unwrap(),
        )
        .unwrap();
    transform_method(
        method,
        &ReflectionData {
            method: IdMethod::from_smali(
                "Lcom/example/theseus/reflection/Reflectee;->transfer(Ljava/lang/String;)Ljava/lang/String;",
            )
            .unwrap(),
        },
    )
    .unwrap();
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
    apk_frauder::replace_dex(
        cli.apk.path.unwrap(),
        cli.out,
        &mut dex_files,
        cli.keystore,
        None::<PathBuf>,
        None::<PathBuf>,
        None::<HashMap<_, Option<Cursor<&[u8]>>>>,
    );
}
