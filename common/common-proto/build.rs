fn main() {
    prost_build::compile_protos(&["../proto/common/common.proto"], &["../proto"])
        .unwrap();
}
