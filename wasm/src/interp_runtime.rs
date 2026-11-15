use chrono::{DateTime, SecondsFormat, Utc};
use rscel::InterpRuntime;
use wasm_bindgen::prelude::*;

/// Per-execution runtime state. Its start time is what `now()` returns during a run.
///
/// Pass the same runtime to several executions to replay them with an identical time.
#[wasm_bindgen(js_name = InterpRuntime)]
#[derive(Clone)]
pub struct WasmInterpRuntime {
    inner: InterpRuntime,
}

impl WasmInterpRuntime {
    pub fn inner(&self) -> &InterpRuntime {
        &self.inner
    }
}

#[wasm_bindgen(js_class = InterpRuntime)]
impl WasmInterpRuntime {
    /// Create a runtime starting at `startTime`, or at the current time if omitted.
    #[wasm_bindgen(constructor)]
    pub fn new(start_time: Option<js_sys::Date>) -> Result<WasmInterpRuntime, JsValue> {
        let inner = match start_time {
            Some(date) => {
                let millis = date.get_time();
                if millis.is_nan() {
                    return Err(JsValue::from_str(
                        "InterpRuntime startTime is an invalid Date",
                    ));
                }

                let start = DateTime::from_timestamp_millis(millis as i64)
                    .ok_or_else(|| JsValue::from_str("InterpRuntime startTime is out of range"))?;
                InterpRuntime::new(start)
            }
            None => InterpRuntime::default(),
        };

        Ok(WasmInterpRuntime { inner })
    }

    /// Create a runtime from an RFC 3339 string, keeping full (nanosecond) precision.
    #[wasm_bindgen(js_name = fromIso)]
    pub fn from_iso(iso: &str) -> Result<WasmInterpRuntime, JsValue> {
        let start = DateTime::parse_from_rfc3339(iso)
            .map_err(|err| JsValue::from_str(&format!("invalid RFC 3339 time '{iso}': {err}")))?;

        Ok(WasmInterpRuntime {
            inner: InterpRuntime::new(start.with_timezone(&Utc)),
        })
    }

    /// The start time as a `Date` (millisecond precision).
    #[wasm_bindgen(getter = startTime)]
    pub fn start_time(&self) -> js_sys::Date {
        let millis = self.inner.start_time().timestamp_millis() as f64;
        js_sys::Date::new(&JsValue::from_f64(millis))
    }

    /// The start time as an RFC 3339 UTC string with nanoseconds (full precision).
    #[wasm_bindgen(getter = startTimeIso)]
    pub fn start_time_iso(&self) -> String {
        self.inner
            .start_time()
            .to_rfc3339_opts(SecondsFormat::Nanos, true)
    }

    /// Whether both runtimes have the same start time.
    pub fn equals(&self, other: &WasmInterpRuntime) -> bool {
        self.inner == other.inner
    }
}
