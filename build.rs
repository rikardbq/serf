use std::env;
use std::fs;
use std::path::Path;

use dirs::home_dir;

fn main() {
    let build_out_dir = env::var_os("OUT_DIR").unwrap();
    // let test = home_dir().unwrap_or(env::var_os("OUT_DIR").unwrap().into());
    let gen_dest_path = Path::new(&build_out_dir).join("gen.rs");
    fs::write(
        &gen_dest_path,
        r#"
        pub const DEFAULT_PORT: u16 = 8080;
        pub const DEFAULT_DB_MAX_CONN: u32 = 12;
        pub const DEFAULT_DB_MAX_IDLE_TIME: u64 = 3600;
        pub fn message() -> &'static str {
            "Hello, World!"
        }
        "#
    ).unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}