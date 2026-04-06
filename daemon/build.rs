// build.rs - compile .po → .mo via msgfmt at build time

use std::path::Path;
use std::process::Command;

fn main() {
    let locales = &["zh_CN"];
    let i18n_dir = Path::new("i18n");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    for locale in locales {
        let po = i18n_dir.join(format!("{locale}.po"));
        let mo = Path::new(&out_dir).join(format!("{locale}.mo"));

        if !po.exists() {
            panic!("missing locale file: {}", po.display());
        }

        let status = Command::new("msgfmt")
            .args([
                po.to_str().unwrap(),
                "-o",
                mo.to_str().unwrap(),
            ])
            .status()
            .expect("failed to run msgfmt - install gettext tools (apt install gettext / brew install gettext)");

        if !status.success() {
            panic!("msgfmt failed for {locale}");
        }
    }

    println!("cargo:rerun-if-changed=i18n/");
}
