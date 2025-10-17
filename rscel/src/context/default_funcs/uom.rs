use rscel_macro::dispatch;

pub use methods::dispatch as uom_convert;

use crate::{CelError, CelResult};
use uom::si::f64::{Mass, ThermodynamicTemperature, Velocity, Volume};
use uom::si::mass::{gram, kilogram, milligram, ounce, pound, slug, ton};
use uom::si::thermodynamic_temperature::{degree_celsius, degree_fahrenheit, kelvin};
use uom::si::velocity::{
    foot_per_second, kilometer_per_hour, knot, meter_per_second, mile_per_hour,
};
use uom::si::volume::{
    cubic_foot, cubic_meter, cubic_yard, cup, fluid_ounce, gallon, liter, milliliter, pint_dry,
    pint_liquid, quart_dry, quart_liquid, tablespoon, teaspoon,
};

const STONE_IN_POUNDS: f64 = 14.0;

#[dispatch]
pub mod methods {
    use crate::{context::default_funcs::uom::uom_convert_internal, CelResult, CelValue};

    fn uom_convert(base: u64, from: String, to: String) -> CelResult<f64> {
        uom_convert_internal(base as f64, &from, &to)
    }

    fn uom_convert(base: i64, from: String, to: String) -> CelResult<f64> {
        uom_convert_internal(base as f64, &from, &to)
    }

    fn uom_convert(base: f64, from: String, to: String) -> CelResult<f64> {
        uom_convert_internal(base as f64, &from, &to)
    }
}

fn uom_convert_internal(base: f64, from: &str, to: &str) -> CelResult<f64> {
    let from_unit = Unit::from_str(from)
        .ok_or_else(|| CelError::argument(&format!("Unsupported unit '{}'.", from)))?;
    let to_unit = Unit::from_str(to)
        .ok_or_else(|| CelError::argument(&format!("Unsupported unit '{}'.", to)))?;

    match (from_unit, to_unit) {
        (Unit::Mass(from_mass), Unit::Mass(to_mass)) => {
            let mass = from_mass.into_mass(base);
            Ok(to_mass.from_mass(mass))
        }
        (Unit::Volume(from_volume), Unit::Volume(to_volume)) => {
            let volume = from_volume.into_volume(base);
            Ok(to_volume.from_volume(volume))
        }
        (Unit::Speed(from_speed), Unit::Speed(to_speed)) => {
            let speed = from_speed.into_velocity(base);
            Ok(to_speed.from_velocity(speed))
        }
        (Unit::Temperature(from_temp), Unit::Temperature(to_temp)) => {
            let temp = from_temp.into_temperature(base);
            Ok(to_temp.from_temperature(temp))
        }
        _ => Err(CelError::argument(&format!(
            "Cannot convert units '{}' -> '{}'.",
            from, to
        ))),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unit {
    Mass(MassUnit),
    Volume(VolumeUnit),
    Speed(SpeedUnit),
    Temperature(TemperatureUnit),
}

impl Unit {
    fn from_str(unit: &str) -> Option<Self> {
        let normalized = unit.trim().to_lowercase();
        let normalized = normalized.trim_matches('°');

        Some(match normalized {
            "kg" | "kilogram" | "kilograms" => Unit::Mass(MassUnit::Kilogram),
            "g" | "gram" | "grams" => Unit::Mass(MassUnit::Gram),
            "mg" | "milligram" | "milligrams" => Unit::Mass(MassUnit::Milligram),
            "lb" | "lbs" | "pound" | "pounds" => Unit::Mass(MassUnit::Pound),
            "oz" | "ounce" | "ounces" => Unit::Mass(MassUnit::Ounce),
            "stone" | "st" | "stones" => Unit::Mass(MassUnit::Stone),
            "slug" | "slugs" => Unit::Mass(MassUnit::Slug),
            "ton" | "tonne" | "metric_ton" | "metric ton" => Unit::Mass(MassUnit::Ton),

            "l" | "liter" | "liters" | "litre" | "litres" => Unit::Volume(VolumeUnit::Liter),
            "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" => {
                Unit::Volume(VolumeUnit::Milliliter)
            }
            "gal" | "gallon" | "gallons" => Unit::Volume(VolumeUnit::Gallon),
            "quart" | "quarts" | "qt" | "qts" | "liquid quart" | "liquid_quart" => {
                Unit::Volume(VolumeUnit::QuartLiquid)
            }
            "dry quart" | "dry_quart" => Unit::Volume(VolumeUnit::QuartDry),
            "pint" | "pints" | "pt" | "pts" | "liquid pint" | "liquid_pint" => {
                Unit::Volume(VolumeUnit::PintLiquid)
            }
            "dry pint" | "dry_pint" => Unit::Volume(VolumeUnit::PintDry),
            "cup" | "cups" => Unit::Volume(VolumeUnit::Cup),
            "fl oz" | "floz" | "fluid ounce" | "fluid_ounce" | "fluid-ounce" => {
                Unit::Volume(VolumeUnit::FluidOunce)
            }
            "tbsp" | "tablespoon" | "tablespoons" => Unit::Volume(VolumeUnit::Tablespoon),
            "tsp" | "teaspoon" | "teaspoons" => Unit::Volume(VolumeUnit::Teaspoon),
            "cubic meter" | "cubic_meter" | "m3" => Unit::Volume(VolumeUnit::CubicMeter),
            "cubic foot" | "cubic_foot" | "ft3" | "cu ft" => Unit::Volume(VolumeUnit::CubicFoot),
            "cubic yard" | "cubic_yard" | "yd3" | "cu yd" => Unit::Volume(VolumeUnit::CubicYard),

            "m/s" | "meter per second" | "meters per second" | "meter_per_second" => {
                Unit::Speed(SpeedUnit::MeterPerSecond)
            }
            "km/h"
            | "kph"
            | "kilometer per hour"
            | "kilometers per hour"
            | "kilometer_per_hour" => Unit::Speed(SpeedUnit::KilometerPerHour),
            "mph" | "mile per hour" | "miles per hour" | "mile_per_hour" => {
                Unit::Speed(SpeedUnit::MilePerHour)
            }
            "kn" | "knot" | "knots" => Unit::Speed(SpeedUnit::Knot),
            "ft/s" | "fps" | "foot per second" | "feet per second" | "foot_per_second" => {
                Unit::Speed(SpeedUnit::FootPerSecond)
            }

            "k" | "kelvin" => Unit::Temperature(TemperatureUnit::Kelvin),
            "c" | "celsius" => Unit::Temperature(TemperatureUnit::Celsius),
            "f" | "fahrenheit" => Unit::Temperature(TemperatureUnit::Fahrenheit),
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MassUnit {
    Kilogram,
    Gram,
    Milligram,
    Pound,
    Ounce,
    Stone,
    Ton,
    Slug,
}

impl MassUnit {
    fn into_mass(self, value: f64) -> Mass {
        match self {
            MassUnit::Kilogram => Mass::new::<kilogram>(value),
            MassUnit::Gram => Mass::new::<gram>(value),
            MassUnit::Milligram => Mass::new::<milligram>(value),
            MassUnit::Pound => Mass::new::<pound>(value),
            MassUnit::Ounce => Mass::new::<ounce>(value),
            MassUnit::Stone => Mass::new::<pound>(value * STONE_IN_POUNDS),
            MassUnit::Ton => Mass::new::<ton>(value),
            MassUnit::Slug => Mass::new::<slug>(value),
        }
    }

    fn from_mass(self, mass: Mass) -> f64 {
        match self {
            MassUnit::Kilogram => mass.get::<kilogram>(),
            MassUnit::Gram => mass.get::<gram>(),
            MassUnit::Milligram => mass.get::<milligram>(),
            MassUnit::Pound => mass.get::<pound>(),
            MassUnit::Ounce => mass.get::<ounce>(),
            MassUnit::Stone => mass.get::<pound>() / STONE_IN_POUNDS,
            MassUnit::Ton => mass.get::<ton>(),
            MassUnit::Slug => mass.get::<slug>(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VolumeUnit {
    Liter,
    Milliliter,
    Gallon,
    QuartLiquid,
    QuartDry,
    PintLiquid,
    PintDry,
    Cup,
    FluidOunce,
    Tablespoon,
    Teaspoon,
    CubicMeter,
    CubicFoot,
    CubicYard,
}

impl VolumeUnit {
    fn into_volume(self, value: f64) -> Volume {
        match self {
            VolumeUnit::Liter => Volume::new::<liter>(value),
            VolumeUnit::Milliliter => Volume::new::<milliliter>(value),
            VolumeUnit::Gallon => Volume::new::<gallon>(value),
            VolumeUnit::QuartLiquid => Volume::new::<quart_liquid>(value),
            VolumeUnit::QuartDry => Volume::new::<quart_dry>(value),
            VolumeUnit::PintLiquid => Volume::new::<pint_liquid>(value),
            VolumeUnit::PintDry => Volume::new::<pint_dry>(value),
            VolumeUnit::Cup => Volume::new::<cup>(value),
            VolumeUnit::FluidOunce => Volume::new::<fluid_ounce>(value),
            VolumeUnit::Tablespoon => Volume::new::<tablespoon>(value),
            VolumeUnit::Teaspoon => Volume::new::<teaspoon>(value),
            VolumeUnit::CubicMeter => Volume::new::<cubic_meter>(value),
            VolumeUnit::CubicFoot => Volume::new::<cubic_foot>(value),
            VolumeUnit::CubicYard => Volume::new::<cubic_yard>(value),
        }
    }

    fn from_volume(self, volume: Volume) -> f64 {
        match self {
            VolumeUnit::Liter => volume.get::<liter>(),
            VolumeUnit::Milliliter => volume.get::<milliliter>(),
            VolumeUnit::Gallon => volume.get::<gallon>(),
            VolumeUnit::QuartLiquid => volume.get::<quart_liquid>(),
            VolumeUnit::QuartDry => volume.get::<quart_dry>(),
            VolumeUnit::PintLiquid => volume.get::<pint_liquid>(),
            VolumeUnit::PintDry => volume.get::<pint_dry>(),
            VolumeUnit::Cup => volume.get::<cup>(),
            VolumeUnit::FluidOunce => volume.get::<fluid_ounce>(),
            VolumeUnit::Tablespoon => volume.get::<tablespoon>(),
            VolumeUnit::Teaspoon => volume.get::<teaspoon>(),
            VolumeUnit::CubicMeter => volume.get::<cubic_meter>(),
            VolumeUnit::CubicFoot => volume.get::<cubic_foot>(),
            VolumeUnit::CubicYard => volume.get::<cubic_yard>(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpeedUnit {
    MeterPerSecond,
    KilometerPerHour,
    MilePerHour,
    FootPerSecond,
    Knot,
}

impl SpeedUnit {
    fn into_velocity(self, value: f64) -> Velocity {
        match self {
            SpeedUnit::MeterPerSecond => Velocity::new::<meter_per_second>(value),
            SpeedUnit::KilometerPerHour => Velocity::new::<kilometer_per_hour>(value),
            SpeedUnit::MilePerHour => Velocity::new::<mile_per_hour>(value),
            SpeedUnit::FootPerSecond => Velocity::new::<foot_per_second>(value),
            SpeedUnit::Knot => Velocity::new::<knot>(value),
        }
    }

    fn from_velocity(self, velocity: Velocity) -> f64 {
        match self {
            SpeedUnit::MeterPerSecond => velocity.get::<meter_per_second>(),
            SpeedUnit::KilometerPerHour => velocity.get::<kilometer_per_hour>(),
            SpeedUnit::MilePerHour => velocity.get::<mile_per_hour>(),
            SpeedUnit::FootPerSecond => velocity.get::<foot_per_second>(),
            SpeedUnit::Knot => velocity.get::<knot>(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TemperatureUnit {
    Kelvin,
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    fn into_temperature(self, value: f64) -> ThermodynamicTemperature {
        match self {
            TemperatureUnit::Kelvin => ThermodynamicTemperature::new::<kelvin>(value),
            TemperatureUnit::Celsius => ThermodynamicTemperature::new::<degree_celsius>(value),
            TemperatureUnit::Fahrenheit => {
                ThermodynamicTemperature::new::<degree_fahrenheit>(value)
            }
        }
    }

    fn from_temperature(self, temp: ThermodynamicTemperature) -> f64 {
        match self {
            TemperatureUnit::Kelvin => temp.get::<kelvin>(),
            TemperatureUnit::Celsius => temp.get::<degree_celsius>(),
            TemperatureUnit::Fahrenheit => temp.get::<degree_fahrenheit>(),
        }
    }
}
