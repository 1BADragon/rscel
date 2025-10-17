use crate::BindContext;

use super::bind_context::RsCelMacro;

mod all;
mod coalesce;
mod exists;
mod exists_one;
mod filter;
mod has;
mod helpers;
mod map;
mod reduce;

pub use all::all_impl;
pub use coalesce::coalesce_impl;
pub use exists::exists_impl;
pub use exists_one::exists_one_impl;
pub use filter::filter_impl;
pub use has::has_impl;
pub use map::map_impl;
pub use reduce::reduce_impl;

const DEFAULT_MACROS: &[(&str, &'static RsCelMacro)] = &[
    ("has", &has_impl),
    ("all", &all_impl),
    ("exists", &exists_impl),
    ("exists_one", &exists_one_impl),
    ("filter", &filter_impl),
    ("map", &map_impl),
    ("reduce", &reduce_impl),
    ("coalesce", &coalesce_impl),
];

const COMPILE_MACROS: &[(&str, &'static RsCelMacro)] = &[
    ("all", &all_impl),
    ("exists", &exists_impl),
    ("exists_one", &exists_one_impl),
    ("filter", &filter_impl),
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
