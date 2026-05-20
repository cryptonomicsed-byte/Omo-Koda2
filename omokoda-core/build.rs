fn main() {
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("failed to find vendored protoc");
    std::env::set_var("PROTOC", protoc);
    println!("cargo:rerun-if-changed=../shared/proto/events.proto");

    prost_build::compile_protos(&["../shared/proto/events.proto"], &["../shared/proto/"]).unwrap();
}
