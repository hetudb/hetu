fn main() {
    // common_building::setup();
    build_proto_test();
}

fn build_proto_test() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .extern_path(".schema", "::schema_proto")
        .compile(
            &["schema.proto", "photon.proto"],
            &["../proto/schema", "../proto/photon"],
        )
        .unwrap();
}
