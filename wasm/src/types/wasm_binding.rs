use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_VALUE: &'static str = r#"
type ICelValue = 
    number 
    | string 
    | {[key: string]: ICelValue} 
    | ICelValue[]
    | null 
    | bigint 
    | {'cel_value': number | bigint} 
    | {'cel_float': number} 
    | {'cel_uint': number | bigint};
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_BINDING: &'static str = r#"
interface ICelBinding {
    [key: string]: ICelValue
};
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ICelValue")]
    pub type ICelValue;

    #[wasm_bindgen(typescript_type = "ICelBinding")]
    pub type ICelBinding;
}
