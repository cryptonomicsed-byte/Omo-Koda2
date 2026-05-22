fn main() {
    // Try to use vendored protoc first, but fall back to system protoc if it fails (e.g. on Android/aarch64)
    let protoc = match protoc_bin_vendored::protoc_bin_path() {
        Ok(path) => path,
        Err(_) => std::path::PathBuf::from("protoc"),
    };

    if let Ok(path_str) = protoc.clone().into_os_string().into_string() {
        std::env::set_var("PROTOC", path_str);
    }

    println!("cargo:rerun-if-changed=../shared/proto/events.proto");

    prost_build::compile_protos(&["../shared/proto/events.proto"], &["../shared/proto/"]).unwrap();
}
