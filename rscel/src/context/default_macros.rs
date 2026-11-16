use crate::BindContext;

use super::bind_context::RsCelMacro;

mod all;
mod bind;
mod coalesce;
mod count;
mod exists;
mod exists_one;
mod filter;
mod find;
mod flat_map;
mod has;
mod has_all;
mod has_any;
mod has_then;
mod map;
mod reduce;

pub use all::all_impl;
pub use bind::bind_impl;
pub use coalesce::coalesce_impl;
pub use count::count_impl;
pub use exists::exists_impl;
pub use exists_one::exists_one_impl;
pub use filter::filter_impl;
pub use find::find_impl;
pub use flat_map::flat_map_impl;
pub use has::has_impl;
pub use has_all::has_all_impl;
pub use has_any::has_any_impl;
pub use has_then::has_then_impl;
pub use map::map_impl;
pub use reduce::reduce_impl;

const DEFAULT_MACROS: &[(&str, &RsCelMacro)] = &[
    ("all", &all_impl),
    ("bind", &bind_impl),
    ("coalesce", &coalesce_impl),
    ("count", &count_impl),
    ("exists", &exists_impl),
    ("exists_one", &exists_one_impl),
    ("filter", &filter_impl),
    ("find", &find_impl),
    ("flatMap", &flat_map_impl),
    ("has", &has_impl),
    ("hasAll", &has_all_impl),
    ("hasThen", &has_then_impl),
    ("hasAny", &has_any_impl),
    ("map", &map_impl),
    ("reduce", &reduce_impl),
];

const COMPILE_MACROS: &[(&str, &RsCelMacro)] = &[
    ("all", &all_impl),
    ("count", &count_impl),
    ("exists", &exists_impl),
    ("exists_one", &exists_one_impl),
    ("filter", &filter_impl),
    ("find", &find_impl),
    ("flatMap", &flat_map_impl),
    ("map", &map_impl),
    ("reduce", &reduce_impl),
];

pub fn load_default_macros(exec_ctx: &mut BindContext) {
    for (name, macro_) in DEFAULT_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}

pub fn load_compile_macros(exec_ctx: &mut BindContext) {
    for (name, macro_) in COMPILE_MACROS.iter() {
        exec_ctx.bind_macro(name, *macro_)
    }
}
