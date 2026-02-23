// format_helpers.rs

use chrono::Duration;
use rscel_macro::dispatch;

use crate::CelValue;

pub use methods::dispatch as format;

#[dispatch]
mod methods {
    use super::{
        apply_float_spec, apply_int_spec, apply_string_spec, bytes_to_hex,
        cel_value_default_format, format_duration, parse_num_spec,
    };
    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime,
        types::{cel_value::CelValueMap, CelBytes},
        CelError, CelResult, CelValue,
    };
    use chrono::{DateTime, Duration, Utc};

    // ── Timestamp ────────────────────────────────────────────────────────────

    fn format(this: DateTime<Utc>, opts: String) -> String {
        this.format(&opts).to_string()
    }
    fn format(this: DateTime<Utc>, timezone: String, opts: String) -> CelResult<String> {
        Ok(get_adjusted_datetime(this, timezone)?
            .format(&opts)
            .to_string())
    }
    fn format(timestamp: DateTime<Utc>, opts: String) -> String {
        timestamp.format(&opts).to_string()
    }
    fn format(timestamp: DateTime<Utc>, timezone: String, opts: String) -> CelResult<String> {
        Ok(get_adjusted_datetime(timestamp, timezone)?
            .format(&opts)
            .to_string())
    }

    // ── Int ──────────────────────────────────────────────────────────────────

    fn format(this: i64) -> CelResult<String> {
        Ok(this.to_string())
    }
    fn format(this: i64, spec: String) -> CelResult<String> {
        Ok(apply_int_spec(
            &parse_num_spec(&spec).map_err(CelError::Argument)?,
            this,
        ))
    }
    fn format(val: i64) -> CelResult<String> {
        Ok(val.to_string())
    }
    fn format(val: i64, spec: String) -> CelResult<String> {
        Ok(apply_int_spec(
            &parse_num_spec(&spec).map_err(CelError::Argument)?,
            val,
        ))
    }

    // ── UInt ─────────────────────────────────────────────────────────────────

    fn format(this: u64) -> CelResult<String> {
        Ok(this.to_string())
    }
    fn format(this: u64, spec: String) -> CelResult<String> {
        Ok(apply_int_spec(
            &parse_num_spec(&spec).map_err(CelError::Argument)?,
            this as i64,
        ))
    }
    fn format(val: u64) -> CelResult<String> {
        Ok(val.to_string())
    }
    fn format(val: u64, spec: String) -> CelResult<String> {
        Ok(apply_int_spec(
            &parse_num_spec(&spec).map_err(CelError::Argument)?,
            val as i64,
        ))
    }

    // ── Float ────────────────────────────────────────────────────────────────

    fn format(this: f64) -> CelResult<String> {
        Ok(this.to_string())
    }
    fn format(this: f64, spec: String) -> CelResult<String> {
        Ok(apply_float_spec(
            &parse_num_spec(&spec).map_err(CelError::Argument)?,
            this,
        ))
    }
    fn format(val: f64) -> CelResult<String> {
        Ok(val.to_string())
    }
    fn format(val: f64, spec: String) -> CelResult<String> {
        Ok(apply_float_spec(
            &parse_num_spec(&spec).map_err(CelError::Argument)?,
            val,
        ))
    }

    // ── Bool ─────────────────────────────────────────────────────────────────

    fn format(this: bool) -> CelResult<String> {
        Ok(if this { "true" } else { "false" }.to_string())
    }
    fn format(this: bool, _spec: String) -> CelResult<String> {
        Ok(if this { "true" } else { "false" }.to_string())
    }
    fn format(val: bool) -> CelResult<String> {
        Ok(if val { "true" } else { "false" }.to_string())
    }
    fn format(val: bool, _spec: String) -> CelResult<String> {
        Ok(if val { "true" } else { "false" }.to_string())
    }

    // ── String ───────────────────────────────────────────────────────────────

    fn format(this: String) -> CelResult<String> {
        Ok(this)
    }
    fn format(this: String, spec: String) -> CelResult<String> {
        apply_string_spec(&this, &spec).map_err(CelError::Argument)
    }
    fn format(val: String) -> CelResult<String> {
        Ok(val)
    }
    fn format(val: String, spec: String) -> CelResult<String> {
        apply_string_spec(&val, &spec).map_err(CelError::Argument)
    }

    // ── Bytes ────────────────────────────────────────────────────────────────

    fn format(this: CelBytes) -> CelResult<String> {
        Ok(bytes_to_hex(this.as_slice()))
    }
    fn format(this: CelBytes, _spec: String) -> CelResult<String> {
        Ok(bytes_to_hex(this.as_slice()))
    }
    fn format(val: CelBytes) -> CelResult<String> {
        Ok(bytes_to_hex(val.as_slice()))
    }
    fn format(val: CelBytes, _spec: String) -> CelResult<String> {
        Ok(bytes_to_hex(val.as_slice()))
    }

    // ── Duration ─────────────────────────────────────────────────────────────

    fn format(this: Duration) -> CelResult<String> {
        format_duration(&this, "").map_err(CelError::Argument)
    }
    fn format(this: Duration, spec: String) -> CelResult<String> {
        format_duration(&this, &spec).map_err(CelError::Argument)
    }
    fn format(val: Duration) -> CelResult<String> {
        format_duration(&val, "").map_err(CelError::Argument)
    }
    fn format(val: Duration, spec: String) -> CelResult<String> {
        format_duration(&val, &spec).map_err(CelError::Argument)
    }

    // ── List ─────────────────────────────────────────────────────────────────

    fn format(this: Vec<CelValue>) -> CelResult<String> {
        let inner = this
            .iter()
            .map(cel_value_default_format)
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("[{inner}]"))
    }
    fn format(this: Vec<CelValue>, _spec: String) -> CelResult<String> {
        let inner = this
            .iter()
            .map(cel_value_default_format)
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("[{inner}]"))
    }
    fn format(val: Vec<CelValue>) -> CelResult<String> {
        let inner = val
            .iter()
            .map(cel_value_default_format)
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("[{inner}]"))
    }
    fn format(val: Vec<CelValue>, _spec: String) -> CelResult<String> {
        let inner = val
            .iter()
            .map(cel_value_default_format)
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("[{inner}]"))
    }

    // ── Map ──────────────────────────────────────────────────────────────────

    fn format(this: CelValueMap) -> CelResult<String> {
        let inner = this
            .iter()
            .map(|(k, v)| format!("{}: {}", k, cel_value_default_format(v)))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("{{{inner}}}"))
    }
    fn format(this: CelValueMap, _spec: String) -> CelResult<String> {
        let inner = this
            .iter()
            .map(|(k, v)| format!("{}: {}", k, cel_value_default_format(v)))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("{{{inner}}}"))
    }
    fn format(val: CelValueMap) -> CelResult<String> {
        let inner = val
            .iter()
            .map(|(k, v)| format!("{}: {}", k, cel_value_default_format(v)))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("{{{inner}}}"))
    }
    fn format(val: CelValueMap, _spec: String) -> CelResult<String> {
        let inner = val
            .iter()
            .map(|(k, v)| format!("{}: {}", k, cel_value_default_format(v)))
            .collect::<Vec<_>>()
            .join(", ");
        Ok(format!("{{{inner}}}"))
    }

    // ── Null + catch-all (Message, Enum, Dyn) ────────────────────────────────
    // The macro maps () → Null and CelValue → catch-all (mangle 'z').
    // Null has no meaningful `this` form in CEL so only the arg variants are
    // needed; the CelValue overloads below will catch everything else including
    // calls where the receiver didn't match any specific type above.

    fn format(_val: ()) -> CelResult<String> {
        Ok("null".to_string())
    }
    fn format(_val: (), _spec: String) -> CelResult<String> {
        Ok("null".to_string())
    }
}

// ─── Numeric spec ─────────────────────────────────────────────────────────────

/// Parsed numeric format spec subset that maps cleanly onto Rust fmt:
///   [0][width][.precision][type]
///
/// Fill is always ' ' (space) or '0' (zero-pad).
/// Supported types: f  e  E  g  x  X  o  b  d  (no type = default Display)
#[derive(Debug, Default)]
pub struct NumSpec {
    pub zero_pad: bool,
    pub width: Option<usize>,
    pub precision: Option<usize>,
    pub ty: char, // '\0' = no type char supplied
}

pub fn parse_num_spec(s: &str) -> Result<NumSpec, String> {
    let mut spec = NumSpec::default();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    // [0]  zero-pad flag
    if i < chars.len() && chars[i] == '0' {
        spec.zero_pad = true;
        i += 1;
    }

    // [width]
    let w_start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    if i > w_start {
        spec.width = Some(
            chars[w_start..i]
                .iter()
                .collect::<String>()
                .parse()
                .unwrap(),
        );
    }

    // [.precision]
    if i < chars.len() && chars[i] == '.' {
        i += 1;
        let p_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i == p_start {
            return Err("Expected digits after '.' in format spec".into());
        }
        spec.precision = Some(
            chars[p_start..i]
                .iter()
                .collect::<String>()
                .parse()
                .unwrap(),
        );
    }

    // [type]
    if i < chars.len() {
        let ty = chars[i];
        if !matches!(
            ty,
            'f' | 'e' | 'E' | 'g' | 'G' | 'x' | 'X' | 'o' | 'b' | 'd'
        ) {
            return Err(format!("Unknown format type '{ty}'"));
        }
        spec.ty = ty;
        i += 1;
    }

    if i != chars.len() {
        return Err(format!(
            "Unexpected trailing chars in format spec at position {i}"
        ));
    }

    Ok(spec)
}

pub fn apply_float_spec(spec: &NumSpec, v: f64) -> String {
    // First produce the raw formatted number without width/padding
    let raw = match spec.ty {
        'e' => match spec.precision {
            Some(p) => format!("{:.prec$e}", v, prec = p),
            None => format!("{:e}", v),
        },
        'E' => match spec.precision {
            Some(p) => format!("{:.prec$E}", v, prec = p),
            None => format!("{:E}", v),
        },
        'g' | 'G' => {
            let prec = spec.precision.unwrap_or(6);
            let exp = if v == 0.0 {
                0
            } else {
                v.abs().log10().floor() as i32
            };
            let use_sci = exp < -4 || exp >= prec as i32;
            let s = if use_sci {
                format!("{:.prec$e}", v, prec = prec.saturating_sub(1))
            } else {
                let s = format!("{:.prec$}", v, prec = prec);
                s.trim_end_matches('0').trim_end_matches('.').to_string()
            };
            if spec.ty == 'G' {
                s.to_uppercase()
            } else {
                s
            }
        }
        'd' => format!("{}", v as i64),
        _ => match spec.precision {
            // 'f' or no type
            Some(p) => format!("{:.prec$}", v, prec = p),
            None => format!("{}", v),
        },
    };

    apply_padding(spec, raw)
}

pub fn apply_int_spec(spec: &NumSpec, v: i64) -> String {
    // Delegate to float for float-type specs
    if matches!(spec.ty, 'f' | 'e' | 'E' | 'g' | 'G') {
        return apply_float_spec(spec, v as f64);
    }

    let raw = match spec.ty {
        'x' => format!("{:x}", v),
        'X' => format!("{:X}", v),
        'o' => format!("{:o}", v),
        'b' => format!("{:b}", v),
        _ => format!("{}", v), // 'd' or no type
    };

    apply_padding(spec, raw)
}

/// Apply width and zero/space padding to an already-formatted string.
fn apply_padding(spec: &NumSpec, raw: String) -> String {
    let Some(width) = spec.width else {
        return raw;
    };
    if raw.len() >= width {
        return raw;
    }
    let pad = width - raw.len();

    if spec.zero_pad {
        // Zero padding goes after a leading sign if present
        if raw.starts_with('-') || raw.starts_with('+') {
            let (sign, rest) = raw.split_at(1);
            format!("{}{}{}", sign, "0".repeat(pad), rest)
        } else {
            format!("{}{}", "0".repeat(pad), raw)
        }
    } else {
        // Space padding — right-align by default
        format!("{}{}", " ".repeat(pad), raw)
    }
}

// ─── String truncation ────────────────────────────────────────────────────────

/// `.N`  → truncate to N chars (no ellipsis)
/// `.Ne` → truncate to N chars, append `…`
/// ``    → return as-is
pub fn apply_string_spec(s: &str, spec: &str) -> Result<String, String> {
    if spec.is_empty() {
        return Ok(s.to_string());
    }
    let inner = spec
        .strip_prefix('.')
        .ok_or_else(|| "String format spec must start with '.'".to_string())?;
    let (num_str, ellipsis) = if let Some(n) = inner.strip_suffix('e') {
        (n, true)
    } else {
        (inner, false)
    };
    let n: usize = num_str
        .parse()
        .map_err(|_| format!("Invalid truncation width '{num_str}'"))?;
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= n {
        return Ok(s.to_string());
    }
    if ellipsis {
        Ok(chars[..n].iter().collect::<String>() + "…")
    } else {
        Ok(chars[..n].iter().collect())
    }
}

// ─── Duration formatting ──────────────────────────────────────────────────────

pub fn format_duration(dur: &Duration, spec: &str) -> Result<String, String> {
    if spec.is_empty() {
        return Ok(default_duration(dur));
    }
    if spec.contains('{') {
        return Ok(format_duration_template(dur, spec));
    }
    format_duration_unit(dur, spec)
}

fn default_duration(dur: &Duration) -> String {
    let s = dur.num_seconds();
    if s.abs() >= 86400 {
        format!("{}d", s / 86400)
    } else if s.abs() >= 3600 {
        format!("{}h", s / 3600)
    } else if s.abs() >= 60 {
        format!("{}m", s / 60)
    } else {
        format!("{}s", s)
    }
}

/// Single-unit: "seconds", "s", "minutes", "m", "hours", "h", "days", "d"
/// Always integer (floor toward zero).
fn format_duration_unit(dur: &Duration, spec: &str) -> Result<String, String> {
    let total_ms = dur.num_milliseconds();
    let (value, label) = match spec.trim() {
        "s" | "sec" | "secs" | "second" | "seconds" => (total_ms / 1_000, "s"),
        "m" | "min" | "mins" | "minute" | "minutes" => (total_ms / 60_000, "m"),
        "h" | "hr" | "hrs" | "hour" | "hours" => (total_ms / 3_600_000, "h"),
        "d" | "day" | "days" => (total_ms / 86_400_000, "d"),
        other => return Err(format!("Unknown duration unit '{other}'")),
    };
    Ok(format!("{value}{label}"))
}

/// Template: any string containing `{d}`, `{h}`, `{m}`, `{s}`.
/// All present placeholders are filled; zero values are always shown.
fn format_duration_template(dur: &Duration, tmpl: &str) -> String {
    let total_secs = dur.num_seconds();
    let neg = total_secs < 0;
    let abs_secs = total_secs.unsigned_abs(); // u64

    let d = (abs_secs / 86400) as i64;
    let h = ((abs_secs % 86400) / 3600) as i64;
    let m = ((abs_secs % 3600) / 60) as i64;
    let s = (abs_secs % 60) as i64;

    let sign = |v: i64| if neg { -v } else { v };

    tmpl.replace("{d}", &sign(d).to_string())
        .replace("{h}", &sign(h).to_string())
        .replace("{m}", &sign(m).to_string())
        .replace("{s}", &sign(s).to_string())
}

// ─── Default CelValue formatter (for List/Map recursive use) ─────────────────

pub fn cel_value_default_format(val: &CelValue) -> String {
    match val {
        CelValue::Int(v) => v.to_string(),
        CelValue::UInt(v) => v.to_string(),
        CelValue::Float(v) => v.to_string(),
        CelValue::Bool(v) => if *v { "true" } else { "false" }.to_string(),
        CelValue::String(v) => v.clone(),
        CelValue::Bytes(v) => bytes_to_hex(v.as_slice()),
        CelValue::Null => "null".to_string(),
        CelValue::Type(v) => v.clone(),
        CelValue::TimeStamp(v) => v.to_rfc3339(),
        CelValue::Duration(v) => default_duration(v),
        CelValue::List(v) => {
            let inner = v
                .iter()
                .map(cel_value_default_format)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{inner}]")
        }
        CelValue::Map(v) => {
            let inner = v
                .iter()
                .map(|(k, v)| format!("{}: {}", k, cel_value_default_format(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{inner}}}")
        }
        #[cfg(feature = "protobuf")]
        CelValue::Message(m) => format!("{m:?}"),
        #[cfg(feature = "protobuf")]
        CelValue::Enum { descriptor, value } => descriptor
            .value_by_number(*value)
            .map(|v| v.name().to_string())
            .unwrap_or_else(|| value.to_string()),
        CelValue::Dyn(v) => format!("{v:?}"),
        _ => String::new(),
    }
}

pub fn bytes_to_hex(b: &[u8]) -> String {
    b.iter().map(|byte| format!("{byte:02x}")).collect()
}
