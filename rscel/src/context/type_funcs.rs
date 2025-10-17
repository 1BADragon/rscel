use crate::{BindContext, CelError, CelValue};

mod bool_type;
mod bytes_type;
mod double_type;
mod duration_type;
mod dyn_type;
mod int_type;
mod string_type;
mod timestamp_type;
mod type_type;
mod uint_type;

pub use bool_type::bool_impl;
pub use bytes_type::bytes_impl;
pub use double_type::double_impl;
pub use duration_type::duration_impl;
pub use dyn_type::dyn_impl;
pub use int_type::int_impl;
pub use string_type::string_impl;
pub use timestamp_type::timestamp_impl;
pub use type_type::type_impl;
pub use uint_type::uint_impl;

pub fn construct_type(type_name: &str, args: Vec<CelValue>) -> CelValue {
    match type_name {
        "bool" => bool_impl(CelValue::from_null(), args),
        "int" => int_impl(CelValue::from_null(), args),
        "uint" => uint_impl(CelValue::from_null(), args),
        "float" => double_impl(CelValue::from_null(), args),
        "double" => double_impl(CelValue::from_null(), args),
        "bytes" => bytes_impl(CelValue::from_null(), args),
        "string" => string_impl(CelValue::from_null(), args),
        "type" => type_impl(CelValue::from_null(), args),
        "timestamp" => timestamp_impl(CelValue::from_null(), args),
        "duration" => duration_impl(CelValue::from_null(), args),
        "dyn" => dyn_impl(CelValue::from_null(), args),
        _ => CelValue::from_err(CelError::runtime(&format!(
            "{} is not constructable",
            type_name
        ))),
    }
}

pub fn load_default_types(bind_ctx: &mut BindContext) {
    bind_ctx.add_type("bool", CelValue::bool_type());
    bind_ctx.add_type("int", CelValue::int_type());
    bind_ctx.add_type("uint", CelValue::uint_type());
    bind_ctx.add_type("float", CelValue::float_type());
    bind_ctx.add_type("double", CelValue::float_type());
    bind_ctx.add_type("string", CelValue::string_type());
    bind_ctx.add_type("bytes", CelValue::bytes_type());
    bind_ctx.add_type("type", CelValue::type_type());
    bind_ctx.add_type("timestamp", CelValue::timestamp_type());
    bind_ctx.add_type("duration", CelValue::duration_type());
    bind_ctx.add_type("null_type", CelValue::null_type());
    bind_ctx.add_type("dyn", CelValue::dyn_type())
}
