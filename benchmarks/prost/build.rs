use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let descriptor_path = out_dir.join("proto_descriptor.bin");

    // Generate prost types + descriptor set for pbjson.
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &[
                "../proto/bench_messages.proto",
                "../proto/benchmarks.proto",
                "../proto/benchmark_message1_proto3.proto",
            ],
            &["../proto/"],
        )
        .expect("failed to compile benchmark protos");

    // Generate serde Serialize/Deserialize impls for proto3-compliant JSON.
    let descriptor_set = std::fs::read(&descriptor_path).expect("failed to read descriptor set");
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)
        .expect("failed to register descriptors")
        .build(&[".bench", ".benchmarks"])
        .expect("failed to build pbjson serde impls");
}
