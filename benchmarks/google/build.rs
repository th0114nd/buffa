fn main() {
    protobuf_codegen::CodeGen::new()
        .inputs([
            "bench_messages.proto",
            "benchmarks.proto",
            "benchmark_message1_proto3.proto",
        ])
        .include("../proto")
        .generate_and_compile()
        .expect("failed to compile benchmark protos");
}
