//
//  Copyright 2021 Google, Inc.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at:
//
//  http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

extern crate protoc_grpcio;
extern crate protoc_rust;

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn paths_to_strs<P: AsRef<Path>>(paths: &[P]) -> Vec<&str> {
    paths.iter().map(|p| p.as_ref().as_os_str().to_str().unwrap()).collect()
}

// Generate mod.rs files for given input files.
fn gen_mod_rs<P: AsRef<Path>>(out_dir: PathBuf, inputs: &[P], grpc: bool) {
    // Will panic if file doesn't exist or it can't create it
    let mut f = fs::File::create(out_dir.join("mod.rs")).unwrap();

    f.write_all(b"// Generated by build.rs\n\n").unwrap();

    for i in 0..inputs.len() {
        let stem = inputs[i].as_ref().file_stem().unwrap();
        f.write_all(format!("pub mod {}; \n", stem.to_str().unwrap()).as_bytes()).unwrap();
        if grpc {
            f.write_all(format!("pub mod {}_grpc;\n", stem.to_str().unwrap()).as_bytes()).unwrap();
        }
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_out_dir = out_dir.join("proto_out");
    let grpc_out_dir = out_dir.join("grpc_out");

    // Make sure to create the output directories before using it
    match fs::create_dir(proto_out_dir.as_os_str().to_str().unwrap()) {
        Err(e) => println!("Proto dir failed to be created: {}", e),
        _ => (),
    };

    match fs::create_dir(grpc_out_dir.as_os_str().to_str().unwrap()) {
        Err(e) => println!("Grpc dir failed to be created: {}", e),
        _ => (),
    }

    // Proto root is //platform2/bt/system
    let proto_root = match env::var("PLATFORM_SUBDIR") {
        Ok(dir) => PathBuf::from(dir).join("bt/system"),
        // Currently at //platform2/bt/system/gd/rust/facade_proto
        Err(_) => {
            PathBuf::from(env::current_dir().unwrap()).join("../../..").canonicalize().unwrap()
        }
    };

    //
    // Generate protobuf output
    //
    let facade_dir = proto_root.join("blueberry/facade");
    let proto_input_files = [facade_dir.join("common.proto")];
    let proto_include_dirs = [facade_dir.clone()];

    protoc_rust::Codegen::new()
        .out_dir(proto_out_dir.as_os_str().to_str().unwrap())
        .inputs(&paths_to_strs(&proto_input_files))
        .includes(&paths_to_strs(&proto_include_dirs))
        .customize(Default::default())
        .run()
        .expect("protoc");

    //
    // Generate grpc output
    //
    let grpc_proto_input_files = [
        facade_dir.join("hci/hci_facade.proto"),
        facade_dir.join("hci/controller_facade.proto"),
        facade_dir.join("hal/hal_facade.proto"),
        facade_dir.join("rootservice.proto"),
    ];
    let grpc_proto_include_dirs =
        [facade_dir.join("hci"), facade_dir.join("hal"), facade_dir, proto_root];

    protoc_grpcio::compile_grpc_protos(
        &grpc_proto_input_files,
        &grpc_proto_include_dirs,
        &grpc_out_dir,
        None,
    )
    .expect("Failed to compile gRPC definitions");

    gen_mod_rs(proto_out_dir, &proto_input_files, false);
    gen_mod_rs(grpc_out_dir, &grpc_proto_input_files, true);
}
