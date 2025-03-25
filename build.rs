use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use prost_build;

// use this step to provide paths that are to be used
// I.E anything provided from the outside in this step will override defaults
// DEFAULTS:
//  (root path) $HOME/.serf/
//  (cfg path) $HOME/.serf/cfg/
//  (users db path) $HOME/.serf/cfg/{hashed}/
//      then {hashed}.db file is the users db
//  (db paths) $HOME/.serf/db/{hashed}/
//      folder is a sha256 hash of the db name
//      containing {hashed}.db file
/**
* let test = home_dir()
       .unwrap_or(env::var_os("OUT_DIR").unwrap().into())
       .as_path();
*/

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    let build_out_dir = env::var_os("OUT_DIR").unwrap();
    let root_dir = env::var_os("SERF_ROOT_DIR").unwrap_or(OsString::from("./.serf"));
    let gen_dest_path = Path::new(&build_out_dir).join("gen.rs");

    let user_db_hash =
        base16ct::lower::encode_string(&Sha256::digest("cfg_root_db_path".as_bytes()));

    fs::write(
        &gen_dest_path,
        format!(
            r#"
            pub const ROOT_DIR: &str = r"{}";
            pub const USER_DB_HASH: &str = r"{}";
            "#,
            root_dir.to_str().unwrap(),
            user_db_hash.as_str()
        ),
    )
    .unwrap();

    let mut prost_build = prost_build::Config::new();
    prost_build.protoc_executable(protoc_bin_vendored::protoc_bin_path().unwrap());
    prost_build.compile_protos(&["src/proto/request.proto"],
                                &["src/proto/"]).unwrap();
}
