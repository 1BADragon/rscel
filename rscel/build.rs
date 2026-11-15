fn main() {
    println!("cargo::rustc-check-cfg=cfg(test_protos)");
    println!("cargo::rerun-if-env-changed=RSCEL_TEST_PROTO");

    // The proto tests need both the generated code and the protobuf runtime, so only
    // generate them when the feature is enabled.
    #[cfg(feature = "protobuf")]
    if std::env::var_os("RSCEL_TEST_PROTO").is_some() {
        println!("cargo::rustc-cfg=test_protos");
        protobuf_codegen::Codegen::new()
            .protoc()
            .include("test/protos")
            .inputs(["test/protos/test.proto"])
            .cargo_out_dir("test_protos")
            .run_from_script();
    }
}
