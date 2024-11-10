use core::str;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use super::state::Usr;

pub const USR_CONFIG_LOCATION: &str = "./config_/usr_";

pub fn usr_config_buffer(location: &str) -> Vec<u8> {
    let mut buffer = Vec::<u8>::new();

    let _ = (
        match File::open(location) {
            Ok(f) => {
                println!("Using usr config from {}", location);
                f
            }
            Err(err) => {
                eprintln!("Error reading usr config with ERROR={}", err);
                panic!()
            }
        }
    ).read_to_end(&mut buffer);

    buffer
}

pub fn parse_usr_config(config_buffer: Vec<u8>) -> HashMap<String, Usr> {
    let usr_config = str::from_utf8(&config_buffer).unwrap();
    let usr_entries: Vec<&str> = usr_config.split("\n").collect();
    let mut usr_map = HashMap::new();

    usr_entries.iter().for_each(|x| {
        let t: Vec<&str> = x.split("|").collect();
        let access_entries: Vec<&str> = t[3].split(",").collect();
        let mut access_map = HashMap::new();

        access_entries.iter().for_each(|y| {
            let u: Vec<&str> = y.split(":").collect();
            let _ = &mut access_map.insert(String::from(u[0]), u[1].parse::<u8>().unwrap());
        });

        let _ = &mut usr_map.insert(String::from(t[1]), Usr {
            u: String::from(t[0]),
            up_hash: String::from(t[2]),
            db_ar: access_map,
        });
    });

    usr_map
}

/*

    // setup user access to be checked later on whenever a client tries to read or write to a given DB
    // re-use setup as part of usr_ conf file update to avoid downtime whenever new entries in access file needs to be handled
    let usr_config = parse_usr_config(USR_CONFIG_LOCATION);
    let usr_entries: Vec<&str> = usr_config.as_str().split("\n").collect();
    let mut usr_map = HashMap::new();

    usr_entries.iter().for_each(|x| {
        let t: Vec<&str> = x.split("|").collect();
        let access_entries: Vec<&str> = t[3].split(",").collect();
        let mut access_map = HashMap::new();

        access_entries.iter().for_each(|y| {
            let u: Vec<&str> = y.split(":").collect();
            let _ = &mut access_map.insert(String::from(u[0]), u[1].parse::<u8>().unwrap());
        });

        let _ = &mut usr_map.insert(String::from(t[1]), Usr {
            u: String::from(t[0]),
            up_hash: String::from(t[2]),
            db_ar: access_map,
        });
    });

    ---

    simply call the usr_ db "./<project_root>/r_/usr_.db" 
    anything else lives in a different folder alltogether "./<project_root>/p_/<whatever>.db"

    

*/
