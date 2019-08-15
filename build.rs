extern crate pb_rs;

use std::path::{Path, PathBuf};
use pb_rs::types::{Config, FileDescriptor};
use std::env;
use pb_rs::ConfigBuilder;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = PathBuf::from("protos");

    let configs = ConfigBuilder::new(&[PathBuf::from("protos/log.proto")], None, Some(&out_dir), &[include_dir]).map_err(|e|{
        println!("{}",e);
    }).unwrap()
        .single_module(false)
        .headers(false)
        .build();

    for ref config in configs{
        FileDescriptor::write_proto(&config).unwrap();
    }
}