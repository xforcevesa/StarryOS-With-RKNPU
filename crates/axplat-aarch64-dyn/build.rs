use std::path::PathBuf;

use serde::Deserialize;

fn main() {
    println!("cargo:rerun-if-env-changed=AX_CONFIG_PATH");
    let config_path =
        std::env::var("AX_CONFIG_PATH").unwrap_or_else(|_| "axconfig.toml".to_string());

    println!("cargo:rerun-if-changed={config_path}");

    println!("cargo:rustc-link-search={}", out_dir().display());
    println!("cargo::rustc-link-arg=-Tlink.x");
    println!("cargo::rustc-link-arg=-no-pie");
    println!("cargo::rustc-link-arg=-znostart-stop-gc");

    let script = "link.ld";

    println!("cargo:rerun-if-changed={script}");
    let mut ld_content = std::fs::read_to_string(script).unwrap();

    let config = std::fs::read_to_string(config_path).unwrap();

    let value: Config = toml::from_str(&config).unwrap();

    ld_content = ld_content.replace("{{SMP}}", &format!("{}", value.plat.cpu_num));

    std::fs::write(out_dir().join("link.x"), ld_content).expect("link.x write failed");
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").unwrap())
}

#[derive(Deserialize)]
struct Config {
    plat: Plat,
}

#[derive(Deserialize)]
struct Plat {
    #[serde(rename = "cpu-num")]
    cpu_num: usize,
}
