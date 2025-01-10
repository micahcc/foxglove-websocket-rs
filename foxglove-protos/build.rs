fn main() {
    // add protos
    let mut protos = vec![];
    let includes = ["protos"];
    for path in std::fs::read_dir("protos/foxglove").expect("Failed to read proto files") {
        if let Ok(path) = &path {
            protos.push(path.path().display().to_string());
            println!("cargo::rerun-if-changed={}", path.path().display());
        }
    }

    let mut prost_config = prost_build::Config::new();
    prost_config.include_file("protos.rs");

    prost_config.type_attribute(".", "#[non_exhaustive]");
    prost_config.extern_path(".google.protobuf.Any", "::prost_wkt_types::Any");
    prost_config.extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp");
    prost_config.extern_path(".google.protobuf.Duration", "::prost_wkt_types::Duration");
    prost_config.extern_path(".google.protobuf.Value", "::prost_wkt_types::Value");
    prost_config.extern_path(".google.protobuf.Struct", "::prost_wkt_types::Struct");

    let descriptor_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("protos.desc");
    prost_reflect_build::Builder::new()
        .descriptor_pool("crate::DESCRIPTOR_POOL")
        .file_descriptor_set_path(&descriptor_path)
        .configure(&mut prost_config, &protos, &includes)
        .expect("Valid reflect config");

    println!("cargo::rerun-if-changed=build.rs");

    prost_config
        .compile_protos(&protos, &includes)
        .expect("Valid prost configuration should have been supplied");
}
