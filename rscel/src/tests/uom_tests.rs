use crate::{BindContext, CelContext, CelError, CelValue};

const EPSILON: f64 = 1e-6;

fn eval_float(expr: &str) -> f64 {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", expr).unwrap();

    match ctx.exec("main", &exec_ctx).unwrap() {
        CelValue::Float(v) => v,
        CelValue::Int(v) => v as f64,
        CelValue::UInt(v) => v as f64,
        other => panic!("Expected numeric result, got {:?}", other),
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= EPSILON,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn converts_mass_units() {
    let pounds = eval_float("uomConvert(1, 'kg', 'lb')");
    assert_close(pounds, 2.204_622_476_037_958_5);

    let kilograms = eval_float("uomConvert(5, 'lb', 'kg')");
    assert_close(kilograms, 2.267_961_85);
}

#[test]
fn converts_volume_units() {
    let liters = eval_float("uomConvert(1, 'gal', 'liter')");
    assert_close(liters, 3.785_412_000_000_000_3);

    let gallons = eval_float("uomConvert(2.5, 'liter', 'gal')");
    assert_close(gallons, 0.660_430_093_210_461_5);
}

#[test]
fn converts_speed_units() {
    let meters_per_second = eval_float("uomConvert(60, 'mph', 'm/s')");
    assert_close(meters_per_second, 26.822_4);

    let miles_per_hour = eval_float("uomConvert(10, 'm/s', 'mph')");
    assert_close(miles_per_hour, 22.369_362_920_544_026);
}

#[test]
fn converts_temperature_units() {
    let fahrenheit = eval_float("uomConvert(0, 'c', 'f')");
    assert_close(fahrenheit, 32.0);

    let kelvin = eval_float("uomConvert(-273.15, 'c', 'k')");
    assert_close(kelvin, 0.0);
}

#[test]
fn rejects_incompatible_units() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", "uomConvert(1, 'kg', 'gal')")
        .unwrap();

    match ctx.exec("main", &exec_ctx) {
        Err(CelError::Argument(msg)) => {
            assert!(msg.contains("Cannot convert"));
        }
        other => panic!("Expected argument error, got {:?}", other),
    }
}

#[test]
fn rejects_unknown_units() {
    let mut ctx = CelContext::new();
    let exec_ctx = BindContext::new();

    ctx.add_program_str("main", "uomConvert(1, 'stone', 'lightyear')")
        .unwrap();

    match ctx.exec("main", &exec_ctx) {
        Err(CelError::Argument(msg)) => {
            assert!(msg.contains("Unsupported unit"));
        }
        other => panic!("Expected argument error, got {:?}", other),
    }
}
