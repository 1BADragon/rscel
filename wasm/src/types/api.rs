use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_INT: &'static str = r#"
export interface CelInt {
    'cel_int': number | bigint;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_UINT: &'static str = r#"
export interface CelUint {
    'cel_uint': number | bigint;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_FLOAT: &'static str = r#"
export interface CelFloat {
    'cel_float': number;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_TYPE: &'static str = r#"
export interface CelType {
    type: 'int' 
        | 'uint'
        | 'float'
        | 'bool'
        | 'string'
        | 'bytes'
        | 'list'
        | 'map'
        | 'null'
        | 'ident'
        | 'type'
        | 'timestamp'
        | 'duration'
        | 'bytecode'
        | 'message'
        | 'enum'
        | 'dyn'
        | 'err'
        ;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_VALUE: &'static str = r#"
export type CelValue =
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
    | CelType
    | undefined;
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_BINDING: &'static str = r#"
export interface CelBinding {
    [key: string]: CelValue;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const ICEL_PROGRAM_DETAILS: &'static str = r#"
export interface CelProgramDetails {
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

    #[wasm_bindgen(typescript_type = "CelProgramDetails")]
    pub type WasmProgramDetails;
}
