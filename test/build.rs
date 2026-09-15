fn main() {
    // Core cel-spec protos
    protobuf_codegen::Codegen::new()
        .protoc()
        .include("protos")
        .inputs([
            "protos/cel/expr/syntax.proto",
            "protos/cel/expr/value.proto",
            "protos/cel/expr/eval.proto",
            "protos/cel/expr/checked.proto",
            "protos/cel/expr/conformance/test/simple.proto",
        ])
        .cargo_out_dir("cel_spec_protos")
        .run_from_script();

    // Proto2 test message types (separate dir to avoid name collision with proto3)
    protobuf_codegen::Codegen::new()
        .protoc()
        .include("protos")
        .inputs([
            "protos/cel/expr/conformance/proto2/test_all_types.proto",
            "protos/cel/expr/conformance/proto2/test_all_types_extensions.proto",
        ])
        .cargo_out_dir("cel_spec_protos_proto2")
        .run_from_script();

    // Proto3 test message types
    protobuf_codegen::Codegen::new()
        .protoc()
        .include("protos")
        .inputs(["protos/cel/expr/conformance/proto3/test_all_types.proto"])
        .cargo_out_dir("cel_spec_protos_proto3")
        .run_from_script();
}
