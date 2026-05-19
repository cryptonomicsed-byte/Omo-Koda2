fn main() {
    prost_build::compile_protos(&["../shared/proto/events.proto"], &["../shared/proto/"]).unwrap();
}
