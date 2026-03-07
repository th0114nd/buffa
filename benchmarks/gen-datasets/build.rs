fn main() {
    buffa_build::Config::new()
        .files(&["../proto/bench_messages.proto", "../proto/benchmarks.proto"])
        .includes(&["../proto/"])
        .compile()
        .expect("failed to compile benchmark protos");
}
