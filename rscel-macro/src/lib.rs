use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemMod};
use types::DispatchMod;

mod types;
mod util;

#[proc_macro_attribute]
pub fn dispatch(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the module with functions
    let input = parse_macro_input!(item as ItemMod);

    DispatchMod::from_mod(input).into_token_stream()
}
