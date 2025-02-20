use std::fs::File;
use std::path::PathBuf;

use androscalpel::{Apk, IdMethod};

use patcher::code_analysis::CFG;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[arg(short, long)]
    apk: PathBuf,
    #[arg(short, long)]
    method: String,
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();
    let mut apk = Apk::load_apk(File::open(&cli.apk).unwrap(), |_, _, _| None, false).unwrap();
    let mid = IdMethod::from_smali(&cli.method).unwrap();
    let class = apk.get_class_mut(&mid.class_).unwrap();
    let method = if let Some(method) = class.virtual_methods.get(&mid) {
        method
    } else {
        class.direct_methods.get(&mid).unwrap()
    };
    let code = method.code.as_ref().unwrap();
    let cfg = CFG::new(code.ins_size.into(), &mid.proto, &code.insns).unwrap();
    print!("{}", cfg.to_dot(&mid.__str__()));
}
