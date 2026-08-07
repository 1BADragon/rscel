mod protos {
    include!(concat!(env!("OUT_DIR"), "/cel_spec_protos/mod.rs"));
}
mod proto2_types {
    include!(concat!(env!("OUT_DIR"), "/cel_spec_protos_proto2/mod.rs"));
}
mod proto3_types {
    include!(concat!(env!("OUT_DIR"), "/cel_spec_protos_proto3/mod.rs"));
}

use protos::eval::expr_value;
use protos::simple::simple_test;
use protos::{simple, value};
use protobuf::reflect::MessageDescriptor;
use regex::Regex;
use rscel::{BindContext, CelContext, CelValue};
use std::collections::HashMap;
use std::sync::OnceLock;

const TEST_DATA_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/cel_spec_tests/simple-test-data");

const SKIP_LIST_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/skip.txt");

fn load_skip_list() -> std::collections::HashSet<String> {
    let content = match std::fs::read_to_string(SKIP_LIST_PATH) {
        Ok(s) => s,
        Err(_) => return Default::default(),
    };
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect()
}

enum Outcome {
    Pass,
    Fail(String),
    Skip(#[allow(dead_code)] String),
}

// Build a name -> MessageDescriptor lookup table from all known generated types.
fn message_registry() -> &'static HashMap<String, MessageDescriptor> {
    static REGISTRY: OnceLock<HashMap<String, MessageDescriptor>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut map = HashMap::new();
        let files = [
            MessageDescriptor::for_type::<proto3_types::test_all_types::TestAllTypes>()
                .file_descriptor()
                .clone(),
            MessageDescriptor::for_type::<proto2_types::test_all_types::TestAllTypes>()
                .file_descriptor()
                .clone(),
        ];
        for file in &files {
            for msg in file.messages() {
                register_message_recursive(msg, &mut map);
            }
        }
        map
    })
}

fn register_message_recursive(
    msg: MessageDescriptor,
    map: &mut HashMap<String, MessageDescriptor>,
) {
    map.insert(msg.full_name().to_string(), msg.clone());
    for nested in msg.nested_messages() {
        register_message_recursive(nested, map);
    }
}

fn main() {
    let filter = std::env::args().nth(1).map(|pat| {
        Regex::new(&pat).unwrap_or_else(|e| {
            eprintln!("invalid filter regex: {e}");
            std::process::exit(2);
        })
    });

    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<(String, String)> = Vec::new();

    let _ = message_registry();
    let skip_list = load_skip_list();

    let mut entries: Vec<_> = std::fs::read_dir(TEST_DATA_DIR)
        .expect("test data dir not found")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |x| x == "binpb"))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();

        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => { eprintln!("ERROR reading {filename}: {e}"); continue; }
        };
        let test_file: simple::SimpleTestFile = match protobuf::Message::parse_from_bytes(&bytes) {
            Ok(f) => f,
            Err(e) => { eprintln!("PARSE ERROR {filename}: {e}"); continue; }
        };

        for section in &test_file.section {
            for test in &section.test {
                let name = format!("{filename}::{}::{}", section.name, test.name);
                total += 1;

                if let Some(ref re) = filter {
                    if !re.is_match(&name) {
                        continue;
                    }
                }

                if skip_list.contains(&name) {
                    skipped += 1;
                    continue;
                }

                let test = test.clone();
                match std::panic::catch_unwind(|| run_test(&test))
                    .unwrap_or_else(|e| {
                        let msg = e
                            .downcast_ref::<&str>()
                            .map(|s| s.to_string())
                            .or_else(|| e.downcast_ref::<String>().cloned())
                            .unwrap_or_else(|| "unknown panic".into());
                        Outcome::Fail(format!("panic: {msg}"))
                    })
                {
                    Outcome::Pass => passed += 1,
                    Outcome::Fail(msg) => {
                        failed += 1;
                        failures.push((name, msg));
                    }
                    Outcome::Skip(_) => skipped += 1,
                }
            }
        }
    }

    if !failures.is_empty() {
        println!("\nFAILURES:");
        for (name, msg) in &failures {
            println!("  FAIL {name}");
            println!("       {msg}");
        }
    }

    println!("\n{total} tests: {passed} passed, {failed} failed, {skipped} skipped");

    if failed > 0 {
        std::process::exit(1);
    }
}

fn run_test(test: &simple::SimpleTest) -> Outcome {
    if test.check_only {
        return Outcome::Skip("check_only".into());
    }

    match &test.result_matcher {
        Some(simple_test::Result_matcher::Unknown(_))
        | Some(simple_test::Result_matcher::AnyUnknowns(_)) => {
            return Outcome::Skip("unknown eval not supported".into());
        }
        _ => {}
    }

    let mut bind_ctx = BindContext::new();
    for (key, expr_value) in &test.bindings {
        match &expr_value.kind {
            Some(expr_value::Kind::Value(v)) => match proto_value_to_cel(v) {
                Ok(cel_val) => bind_ctx.bind_param(key, cel_val),
                Err(e) => return Outcome::Skip(format!("unsupported binding type: {e}")),
            },
            _ => return Outcome::Skip("error/unknown binding not supported".into()),
        }
    }

    let mut ctx = CelContext::new();
    if let Err(e) = ctx.add_program_str("entry", &test.expr) {
        return match &test.result_matcher {
            Some(simple_test::Result_matcher::EvalError(_))
            | Some(simple_test::Result_matcher::AnyEvalErrors(_)) => Outcome::Pass,
            _ => Outcome::Fail(format!("compile error: {e}")),
        };
    }

    let result = ctx.exec("entry", &bind_ctx);

    match &test.result_matcher {
        None => match result {
            Ok(CelValue::Bool(true)) => Outcome::Pass,
            Ok(got) => Outcome::Fail(format!("expected true, got {got:?}")),
            Err(e) => Outcome::Fail(format!("unexpected error: {e}")),
        },

        Some(simple_test::Result_matcher::Value(expected_proto)) => match result {
            Ok(got) => match proto_value_to_cel(expected_proto) {
                Ok(expected) => {
                    if got == expected {
                        Outcome::Pass
                    } else {
                        Outcome::Fail(format!("expected {expected:?}, got {got:?}"))
                    }
                }
                Err(e) => Outcome::Skip(format!("unsupported expected value type: {e}")),
            },
            Err(e) => Outcome::Fail(format!("unexpected error: {e}")),
        },

        Some(simple_test::Result_matcher::TypedResult(tr)) => match result {
            Ok(got) => {
                match proto_value_to_cel(tr.result.as_ref().unwrap_or(&value::Value::new())) {
                    Ok(expected) => {
                        if got == expected {
                            Outcome::Pass
                        } else {
                            Outcome::Fail(format!("expected {expected:?}, got {got:?}"))
                        }
                    }
                    Err(e) => Outcome::Skip(format!("unsupported expected value type: {e}")),
                }
            }
            Err(e) => Outcome::Fail(format!("unexpected error: {e}")),
        },

        Some(simple_test::Result_matcher::EvalError(_))
        | Some(simple_test::Result_matcher::AnyEvalErrors(_)) => match result {
            Err(_) => Outcome::Pass,
            Ok(got) => Outcome::Fail(format!("expected error, got {got:?}")),
        },

        Some(simple_test::Result_matcher::Unknown(_))
        | Some(simple_test::Result_matcher::AnyUnknowns(_)) => unreachable!(),
    }
}

fn proto_value_to_cel(v: &value::Value) -> Result<CelValue, String> {
    match &v.kind {
        Some(value::value::Kind::NullValue(_)) => Ok(CelValue::Null),
        Some(value::value::Kind::BoolValue(b)) => Ok(CelValue::from(*b)),
        Some(value::value::Kind::Int64Value(i)) => Ok(CelValue::from(*i)),
        Some(value::value::Kind::Uint64Value(u)) => Ok(CelValue::from_uint(*u)),
        Some(value::value::Kind::DoubleValue(d)) => Ok(CelValue::from(*d)),
        Some(value::value::Kind::StringValue(s)) => Ok(CelValue::from(s.as_str())),
        Some(value::value::Kind::BytesValue(b)) => Ok(CelValue::from(b.as_slice())),
        Some(value::value::Kind::ListValue(l)) => {
            let items: Result<Vec<CelValue>, _> =
                l.values.iter().map(proto_value_to_cel).collect();
            Ok(CelValue::from_list(items?))
        }
        Some(value::value::Kind::MapValue(m)) => {
            let mut map = HashMap::new();
            for entry in &m.entries {
                let k = entry.key.as_ref().ok_or("map entry missing key")?;
                let v = entry.value.as_ref().ok_or("map entry missing value")?;
                let key_str = match &k.kind {
                    Some(value::value::Kind::StringValue(s)) => s.clone(),
                    Some(value::value::Kind::Int64Value(i)) => i.to_string(),
                    Some(value::value::Kind::Uint64Value(u)) => u.to_string(),
                    Some(value::value::Kind::BoolValue(b)) => b.to_string(),
                    _ => return Err("unsupported map key type".into()),
                };
                map.insert(key_str, proto_value_to_cel(v)?);
            }
            Ok(CelValue::from_map(map))
        }
        Some(value::value::Kind::TypeValue(t)) => Ok(CelValue::from_type(t)),
        Some(value::value::Kind::EnumValue(_)) => Err("enum value".into()),
        Some(value::value::Kind::ObjectValue(any)) => {
            let type_name = any
                .type_url
                .strip_prefix("type.googleapis.com/")
                .unwrap_or(&any.type_url);

            let descriptor = message_registry()
                .get(type_name)
                .ok_or_else(|| format!("unknown message type: {type_name}"))?;

            let mut msg = descriptor.new_instance();
            msg.merge_from_bytes_dyn(&any.value)
                .map_err(|e| format!("failed to decode Any bytes for {type_name}: {e}"))?;

            Ok(CelValue::from_proto_msg(msg))
        }
        None => Err("empty value".into()),
    }
}
