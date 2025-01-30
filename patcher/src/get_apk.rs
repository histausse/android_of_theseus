use androscalpel::Apk;
use clap::Args;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Args, Debug)]
pub struct ApkLocation {
    #[arg(short, long, conflicts_with = "sha256")]
    pub path: Option<PathBuf>,
    #[arg(long, conflicts_with = "path", requires = "androzoo_key")]
    pub sha256: Option<String>,
    #[command(flatten)]
    pub androzoo_key: Option<AndrozooKey>,
}

impl ApkLocation {
    pub fn get_id(&self) -> String {
        match self {
            ApkLocation {
                path: Some(path), ..
            } => path.as_path().file_name().unwrap().to_str().unwrap().into(),
            ApkLocation {
                sha256: Some(sha256),
                ..
            } => sha256.clone(),
            _ => panic!("Invalid ApkLocation"),
        }
    }
}

#[derive(Clone, Args, Debug)]
pub struct AndrozooKey {
    #[arg(long, group = "androzoo_key", conflicts_with = "api_key")]
    api_key_path: Option<PathBuf>,
    #[arg(long, group = "androzoo_key", conflicts_with = "api_key_path")]
    api_key: Option<String>,
}

impl AndrozooKey {
    fn get_key(&self) -> String {
        match self {
            AndrozooKey {
                api_key_path: Some(path),
                ..
            } => read_to_string(path)
                .expect("Failed to read key from file")
                .trim()
                .to_string(),
            AndrozooKey {
                api_key: Some(key), ..
            } => key.trim().to_string(),
            _ => panic!("No key here"),
        }
    }
}

pub fn get_apk(location: &ApkLocation) -> Apk {
    match location {
        ApkLocation {
            androzoo_key: Some(key),
            sha256: Some(sha256),
            ..
        } => {
            let key = key.get_key();
            let url = reqwest::Url::parse_with_params(
                "https://androzoo.uni.lu/api/download",
                &[("apikey", key), ("sha256", sha256.clone())],
            )
            .expect("Failed to parse url");
            let res = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .expect("Failed to build client")
                .get(url)
                .send()
                .expect("Failed to download apk");
            match res.status() {
                reqwest::StatusCode::OK => (),
                s => panic!("Failed to download apk: {:?}", s),
            }
            Apk::load_apk_bin(&res.bytes().expect("Failed to get APK bytes"), false, false).unwrap()
        }
        ApkLocation {
            path: Some(path), ..
        } => Apk::load_apk(path.into(), false, false).unwrap(),
        _ => panic!("Don't know what to do with:\n{:#?}", location),
    }
}
