// Copyright 2021 HetuDB.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::path::Path;

fn main() {
    // common_building::setup();
    build_proto();
}

fn build_proto() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env variable unset");

    println!("manifest_dir: {}", manifest_dir);
    let proto_dir_common = Path::new(&manifest_dir).join("../proto/common");
    let proto_dir_schema = Path::new(&manifest_dir).join("../proto/schema");
    let proto_dir_security = Path::new(&manifest_dir).join("../proto/security");
    let proto_dir_meta = Path::new(&manifest_dir).join("../proto/meta");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .extern_path(".common", "::common_proto")
        .extern_path(".security", "::security_proto")
        .extern_path(".schema", "::schema_proto")
        .compile(
            &[
                "common.proto",
                "schema.proto",
                "security.proto",
                "meta.proto",
            ],
            &[
                &proto_dir_common,
                &proto_dir_schema,
                &proto_dir_security,
                &proto_dir_meta,
            ],
        )
        .unwrap();
}
