use wasm_bindgen::{convert::FromWasmAbi, JsValue};

pub fn generic_of_js<T: FromWasmAbi<Abi = u32>>(
    js: JsValue,
    classname: &str,
) -> Result<T, JsValue> {
    use js_sys::{Object, Reflect};
    let ctor_name = Object::get_prototype_of(&js).constructor().name();
    if ctor_name == classname {
        let ptr = Reflect::get(&js, &JsValue::from_str("__wbg_ptr"))?;
        let ptr_u32: u32 = ptr.as_f64().ok_or(JsValue::NULL)? as u32;
        let foo = unsafe { T::from_abi(ptr_u32) };
        Ok(foo)
    } else {
        Err(JsValue::NULL)
    }
}

#[allow(dead_code)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
