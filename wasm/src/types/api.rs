use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_INT: &'static str = r#"
interface CelInt {
    'cel_int': number | bigint;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_UINT: &'static str = r#"
interface CelUint {
    'cel_uint': number | bigint;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_FLOAT: &'static str = r#"
interface CelFloat {
    'cel_float': number;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_TYPE: &'static str = r#"
interface CelType {
    type: 'string' | 'int' | 'uint' | 'bool' | 'date' | 'duration' | 'null' | 'float'
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_VALUE: &'static str = r#"
type CelValue =
    number 
    | string 
    | bigint 
    | boolean
    | null 
    | CelInt 
    | CelUint 
    | CelFloat
    | {[key: string]: CelValue}
    | CelValue[]
    | CelType;
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_BINDING: &'static str = r#"
interface CelBinding {
    [key: string]: CelValue;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_EVAL_ERROR: &'static str = r#"
interface CelEvalError {
    kind: string,
    msg: string,
    err: any
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_PROGRAM_DETAILS: &'static str = r#"
interface CelProgramDetails {
    source?: string;
    params: string[];
    ast?: any;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CelValue")]
    pub type WasmCelValue;

    #[wasm_bindgen(typescript_type = "CelBinding")]
    pub type WasmCelBinding;

    #[wasm_bindgen(typescript_type = "CelEvalError")]
    pub type WasmEvalError;

    #[wasm_bindgen(typescript_type = "CelProgramDetails")]
    pub type WasmProgramDetails;
}
