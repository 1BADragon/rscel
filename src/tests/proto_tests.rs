use protobuf::MessageField;
use test_case::test_case;

use crate::{BindContext, CelContext, CelValue};

mod protos {
    include!(concat!(env!("OUT_DIR"), "/test_protos/mod.rs"));
}

#[test_case("p.x", 3.into(); "message access")]
#[test_case("c.unitialized_field == 0", true.into(); "unitialized field access")]
#[test_case("c.enum_field == 1", true.into(); "enum int eq")]
#[test_case("c.enum_field == 'FIELD2'", true.into(); "enum str eq")]
#[test_case("c.nested_field.unitialized_field == 0", true.into(); "nexted unitialized field access")]
#[test_case("c.oneof_field1 == 45", true.into(); "oneof field access")]
fn proto_test(prog: &str, res: CelValue) {
    let mut ctx = CelContext::new();
    let mut exec_ctx = BindContext::new();

    let mut p = Box::new(protos::test::Point::new());
    p.x = 3;
    p.y = 4;
    exec_ctx.bind_param_proto_msg("p", p);

    let mut c = Box::new(protos::test::TestMessage1::new());
    c.initialized_field = 7;
    c.enum_field = protos::test::MyEnum::FIELD2.into();
    let mut nested_c = protos::test::TestMessage1::new();
    nested_c.initialized_field = 8;
    c.nested_field = MessageField::some(nested_c);
    c.set_oneof_field1(45);
    exec_ctx.bind_param_proto_msg("c", c);

    ctx.add_program_str("entry", prog)
        .expect("Failed to compile prog");

    assert_eq!(
        ctx.exec("entry", &exec_ctx).expect("failed to run prog"),
        res
    );
}
