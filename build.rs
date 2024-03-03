fn main() {
    protobuf_codegen::Codegen::new()
        .protoc()
        .include("test/protos")
        .inputs(["test/protos/test.proto"])
        .cargo_out_dir("test_protos")
        .run_from_script();
}
