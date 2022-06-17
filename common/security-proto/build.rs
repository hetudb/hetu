fn main() {
    prost_build::compile_protos(&["../proto/security/security.proto"], &["../proto"])
        .unwrap();
}
