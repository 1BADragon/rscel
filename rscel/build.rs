use std::env;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(test_protos)");
    println!("cargo:rerun-if-env-changed=RSCEL_TEST_PROTO");

    if env::var("RSCEL_TEST_PROTO").is_ok() {
        println!("cargo:rustc-cfg=test_protos");
        println!("cargo:rustc-cfg=protobuf");
        protobuf_codegen::Codegen::new()
            .protoc()
            .include("test/protos")
            .inputs(["test/protos/test.proto"])
            .cargo_out_dir("test_protos")
            .run_from_script();
    }
}
