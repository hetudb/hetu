fn main() {
    prost_build::compile_protos(&["../proto/schema/schema.proto"], &["../proto"])
        .unwrap();
}
