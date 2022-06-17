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
