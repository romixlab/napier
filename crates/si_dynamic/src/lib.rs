//! SI quantities and units parsed from text at run time: `4.7 uF`, `10mV`, `4k7` (resistance), `ppm`, `km²`.
//!
//! The unit model follows ww_si (wire_weaver): a [Unit] is a [Prefix], a [BaseUnit] and an exponent; [UnitExpr] is
//! a product, quotient or power of units; every unit has a [Dim] (SI base-unit exponents) and a [Scale] to coherent
//! SI (the gram is 10⁻³ kg, °C is affine). Units beyond the SI set are registered at run time in a [Registry].

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod dim;
mod parse;
mod registry;
mod unit;

pub use dim::{Dim, Scale};
pub use registry::{NamedUnit, Registry};
pub use unit::{BaseUnit, KNOWN_UNITS, Prefix, Unit, UnitExpr};

use parse::{Rule, SIParser, expected_rule, parse_prefix};
use pest::Parser;
use pest::iterators::Pair;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

/// A number and its unit, as parsed: the number keeps its text (`4.7`, `-1.5e3`), so nothing is lost before the
/// caller converts it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Quantity {
    pub number: String,
    pub unit: UnitExpr,
}

impl Quantity {
    /// A number with an optional unit, with only the built-in words; [Registry::parse_quantity] knows more.
    pub fn parse(s: impl AsRef<str>) -> Result<Quantity, Error> {
        parse::quantity(s.as_ref(), &Registry::new())
    }

    /// The number as written, in the unit with its prefixes.
    pub fn value_f64(&self) -> Result<f64, Error> {
        Ok(self.number.replace('\u{2212}', "-").parse::<f64>()?)
    }

    /// The value in the unit without prefixes, as exact decimal text: `4.7 kΩ` is `4700` (Ω), `1 km²` is `1000000`
    /// (m²), `10 kg` is `10000` (g), `50 ppm` is `0.00005`.
    pub fn base_decimal(&self) -> Result<String, Error> {
        parse::shift_decimal(&self.number, self.unit.prefix_exp10())
            .ok_or_else(|| Error::Internal(format!("bad number {:?}", self.number)))
    }

    /// [base_decimal](Self::base_decimal) as an integer, `None` when it has a fraction or doesn't fit.
    pub fn base_i64(&self) -> Result<Option<i64>, Error> {
        Ok(self.base_decimal()?.parse::<i64>().ok())
    }

    /// The value in the unit without prefixes (the prefix raised to the unit's exponent): `1 km²` is 1e6 (m²).
    pub fn as_f64(&self) -> Result<f64, Error> {
        Ok(self.base_decimal()?.parse::<f64>()?)
    }

    /// Like [as_f64](Self::as_f64), in `f32`.
    pub fn as_f32(&self) -> Result<f32, Error> {
        Ok(self.base_decimal()?.parse::<f32>()?)
    }

    /// The value in coherent SI through the unit's scale: `10 g` is 0.01 (kg), `25 °C` is 298.15 (K); fails for an
    /// unknown word and for °C inside a compound unit.
    pub fn to_si_f64(&self) -> Result<f64, Error> {
        self.unit
            .to_si_f64(self.value_f64()?)
            .ok_or_else(|| Error::Internal(format!("{} doesn't convert to SI", self.unit)))
    }
}

/// Resistance in ohms, from resistor notation: `4k7`, `499kR`, `12 kOhms`, `1 mΩ`.
pub struct OhmF32(pub f32);

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Pest(Box<pest::error::Error<Rule>>),
    Float(ParseFloatError),
    Int(ParseIntError),
    /// A unit can't be registered: the word is an SI symbol or stands for another unit.
    Registry(String),
    Internal(String),
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error::Pest(Box::new(e))
    }
}

impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Self {
        Error::Float(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::Int(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Pest(e) => write!(f, "{}", e),
            Error::Float(e) => write!(f, "{}", e),
            Error::Int(e) => write!(f, "{}", e),
            Error::Registry(e) => write!(f, "{}", e),
            Error::Internal(e) => write!(f, "Internal: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl OhmF32 {
    pub fn parse(s: impl AsRef<str>) -> Result<OhmF32, Error> {
        let p = SIParser::parse(Rule::resistance, s.as_ref())?;
        let mut resistance = p
            .into_iter()
            .next()
            .ok_or(expected_rule("resistance"))?
            .into_inner();
        let value = resistance
            .next()
            .ok_or(expected_rule("number_prefix_allowed"))?;
        let value = parse_any_number_f32(value)?;

        if resistance.peek().map(|p| p.as_rule()) == Some(Rule::EOI) {
            return Ok(OhmF32(value));
        }
        let p = resistance
            .next()
            .ok_or(expected_rule("ohm_unit"))?
            .into_inner()
            .next()
            .ok_or(expected_rule("prefix or ohm_or_r"))?;
        let prefix = match p.as_rule() {
            Rule::r_prefix => parse_prefix(p)?,
            Rule::ohm_or_r => Prefix::Unit,
            _ => return Err(Error::Internal("wrong resistance rule".to_owned())),
        };

        Ok(OhmF32(value * prefix.as_f32()))
    }
}

/// `1.5e3`, or resistor notation with the prefix as the decimal point (`4k7` is 4.7 kilo), read as decimal text
/// and shifted exactly before the one conversion to `f32` (RES-2).
fn parse_any_number_f32(num: Pair<Rule>) -> Result<f32, Error> {
    if !matches!(num.as_rule(), Rule::number | Rule::r_number) {
        return Err(Error::Internal(
            "expected number or number_prefix_allowed".into(),
        ));
    }
    let text = num.as_str();
    let mut p = num.into_inner();
    let int = p.next().ok_or(expected_rule("integer_number"))?.as_str();
    let (decimal, exp10) = match p.next() {
        Some(prefix) if prefix.as_rule() == Rule::r_prefix => {
            let exp10 = parse_prefix(prefix)?.exp10() as i32;
            let frac = p.next().map(|f| f.as_str()).unwrap_or("0");
            (format!("{int}.{frac}"), exp10)
        }
        _ => (text.to_string(), 0),
    };
    let decimal = parse::shift_decimal(&decimal, exp10)
        .ok_or_else(|| Error::Internal(format!("bad number {text:?}")))?;
    Ok(decimal.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    /// The single unit of a parsed quantity.
    fn unit_of(q: &Quantity) -> &Unit {
        q.unit.as_unit().expect("a single unit")
    }

    #[test]
    fn si_number_voltage() {
        let num = Quantity::parse("10V").unwrap();
        assert_eq!(num.number, "10");
        assert_eq!(unit_of(&num).prefix, Prefix::Unit);
        assert_eq!(unit_of(&num).base, BaseUnit::Volt);
        assert_eq!(unit_of(&num).exp, 1);
        assert!(approx_eq!(f32, num.as_f32().unwrap(), 10.0, ulps = 2));

        let num = Quantity::parse("10mV").unwrap();
        assert_eq!(num.number, "10");
        assert_eq!(unit_of(&num).prefix, Prefix::Milli);
        assert_eq!(unit_of(&num).base, BaseUnit::Volt);
        assert_eq!(unit_of(&num).exp, 1);
        assert!(approx_eq!(f32, num.as_f32().unwrap(), 0.01, ulps = 2));

        let num = Quantity::parse("10mVDC").unwrap();
        assert_eq!(num.number, "10");
        assert_eq!(unit_of(&num).prefix, Prefix::Milli);
        assert_eq!(unit_of(&num).base.name(), "VDC");
    }

    #[test]
    fn si_number_capacitance() {
        let num = Quantity::parse("4.7 uF").unwrap();
        assert_eq!(num.number, "4.7");
        assert_eq!(unit_of(&num).prefix, Prefix::Micro);
        assert_eq!(unit_of(&num).base, BaseUnit::Farad);
        assert_eq!(unit_of(&num).exp, 1);
        assert!(approx_eq!(f32, num.as_f32().unwrap(), 4.7e-6, ulps = 2));

        let num = Quantity::parse("4.7 µF").unwrap();
        assert_eq!(num.number, "4.7");
        assert_eq!(unit_of(&num).prefix, Prefix::Micro);
        assert_eq!(unit_of(&num).base, BaseUnit::Farad);
        assert_eq!(unit_of(&num).exp, 1);
        assert!(approx_eq!(f32, num.as_f32().unwrap(), 4.7e-6, ulps = 2));
    }

    #[test]
    fn gram_is_the_base_of_mass() {
        // UNIT-6: 10 g was 10 kg, 10 kg was kilo-kilogram
        let q = Quantity::parse("10 g").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Unit, BaseUnit::Gram, 1));
        assert_eq!(q.as_f64().unwrap(), 10.0);
        assert!(approx_eq!(f64, q.to_si_f64().unwrap(), 0.01, ulps = 2));

        let q = Quantity::parse("10 kg").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Kilo, BaseUnit::Gram, 1));
        assert_eq!(q.as_f64().unwrap(), 10_000.0);
        assert_eq!(q.to_si_f64().unwrap(), 10.0);
        assert_eq!(q.unit.to_string(), "kg");
        assert_eq!(q.unit.exp10(), Some(0));
        assert_eq!(q.unit.dim(), Some(Dim::new(0, 0, 1, 0, 0, 0, 0)));
    }

    #[test]
    fn celsius_is_affine() {
        // UNIT-7
        let q = Quantity::parse("25 °C").unwrap();
        assert_eq!(unit_of(&q).base, BaseUnit::DegreeCelsius);
        assert_eq!(q.as_f64().unwrap(), 25.0);
        assert!(approx_eq!(f64, q.to_si_f64().unwrap(), 298.15, ulps = 2));
        let kelvin = UnitExpr::from(BaseUnit::Kelvin);
        let celsius = UnitExpr::from(BaseUnit::DegreeCelsius);
        assert!(approx_eq!(
            f64,
            celsius.convert_f64(-40.0, &kelvin).unwrap(),
            233.15,
            ulps = 2
        ));
        assert!(approx_eq!(
            f64,
            kelvin.convert_f64(0.0, &celsius).unwrap(),
            -273.15,
            ulps = 2
        ));
        assert_eq!(celsius.exp10(), None);
        // a coefficient per °C is a quotient: no offset there
        let tc = Quantity::parse("50 ppm").unwrap().unit / (celsius.clone());
        assert!(tc.has_offset());
        assert_eq!(tc.offset(), 0.0);
        assert_eq!(tc.to_si_f64(1.0), None);
        assert_eq!(tc.dim(), Some(Dim::new(0, 0, 0, 0, -1, 0, 0)));
        assert_eq!(tc.to_string(), "ppm/°C");
    }

    #[test]
    fn ratios() {
        let q = Quantity::parse("5%").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Centi, BaseUnit::Ratio, 1));
        assert_eq!(q.base_decimal().unwrap(), "0.05");
        assert_eq!(q.unit.to_string(), "%");

        let q = Quantity::parse("2 ‰").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Milli, BaseUnit::Ratio, 1));
        assert_eq!(q.unit.to_string(), "‰");

        let q = Quantity::parse("50ppm").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Micro, BaseUnit::Ratio, 1));
        assert_eq!(q.base_decimal().unwrap(), "0.00005");
        assert_eq!(q.unit.to_string(), "ppm");
        assert_eq!(q.unit.dim(), Some(Dim::ONE));

        let q = Quantity::parse("10").unwrap();
        assert_eq!(q.unit, UnitExpr::one());
        assert_eq!(q.unit.to_string(), "");
        // ppm converts to % by a power of ten
        let percent = Quantity::parse("1%").unwrap().unit;
        let ppm = Quantity::parse("1ppm").unwrap().unit;
        assert!(percent.same_but_prefixes(&ppm));
        assert!(approx_eq!(
            f64,
            ppm.convert_f64(10_000.0, &percent).unwrap(),
            1.0,
            ulps = 2
        ));
    }

    #[test]
    fn prefix_raised_to_the_exponent() {
        // NUM-3: 1 km^2 was 1000
        let q = Quantity::parse("1 km^2").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Kilo, BaseUnit::Meter, 2));
        assert_eq!(q.as_f64().unwrap(), 1e6);
        assert_eq!(q.as_f32().unwrap(), 1e6);
        assert_eq!(q.unit.dim(), Some(Dim::new(0, 2, 0, 0, 0, 0, 0)));

        let q = Quantity::parse("2 cm³").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Centi, BaseUnit::Meter, 3));
        assert_eq!(q.base_decimal().unwrap(), "0.000002");

        let q = Quantity::parse("5 ms⁻¹").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Milli, BaseUnit::Second, -1));
        assert_eq!(q.as_f64().unwrap(), 5000.0);
    }

    #[test]
    fn f64_and_exact_values() {
        // NUM-5
        let q = Quantity::parse("13.7 kV").unwrap();
        assert_eq!(q.value_f64().unwrap(), 13.7);
        assert_eq!(q.base_decimal().unwrap(), "13700");
        assert_eq!(q.base_i64().unwrap(), Some(13700));
        assert_eq!(q.as_f64().unwrap(), 13700.0);

        let q = Quantity::parse("4.7 uF").unwrap();
        assert_eq!(q.base_decimal().unwrap(), "0.0000047");
        assert_eq!(q.base_i64().unwrap(), None);
        assert_eq!(q.as_f64().unwrap(), 4.7e-6);

        let q = Quantity::parse("−2.5e-3 MV").unwrap();
        assert_eq!(q.base_decimal().unwrap(), "-2500");
        assert_eq!(q.value_f64().unwrap(), -2.5e-3);

        let q = Quantity::parse("123456789012345678 nV").unwrap();
        assert_eq!(q.base_decimal().unwrap(), "123456789.012345678");
    }

    #[test]
    fn shift_decimal() {
        use crate::parse::shift_decimal;
        assert_eq!(shift_decimal("1", 0).as_deref(), Some("1"));
        assert_eq!(shift_decimal("0", 6).as_deref(), Some("0"));
        assert_eq!(shift_decimal("-0.0", 3).as_deref(), Some("0"));
        assert_eq!(shift_decimal("1.5e3", -1).as_deref(), Some("150"));
        assert_eq!(shift_decimal("100", -3).as_deref(), Some("0.1"));
        assert_eq!(shift_decimal("0.010", 1).as_deref(), Some("0.1"));
        assert_eq!(shift_decimal("abc", 1), None);
    }

    #[test]
    fn dimensions_and_scales() {
        let v = UnitExpr::from(BaseUnit::Volt);
        let a = UnitExpr::from(BaseUnit::Ampere);
        let ohm = UnitExpr::from(BaseUnit::Ohm);
        assert_eq!((v.clone() / a.clone()).dim(), ohm.dim());
        assert_eq!((v.clone() * a).dim(), BaseUnit::Watt.dim());
        assert_eq!(v.dim().unwrap().to_string(), "kg·m²·s⁻³·A⁻¹");

        let slew = UnitExpr::from(Unit::new(Prefix::Milli, BaseUnit::Volt, 1))
            / (Unit::new(Prefix::Micro, BaseUnit::Second, 1));
        assert_eq!(slew.to_string(), "mV/μs");
        assert_eq!(slew.exp10(), Some(3));
        assert_eq!(slew.prefix_exp10(), 3);
        assert!(approx_eq!(f64, slew.factor().unwrap(), 1e3, ulps = 2));
        assert_eq!(slew.without_prefixes().to_string(), "V/s");
        let squared = slew.clone().pow(2);
        assert_eq!(squared.to_string(), "(mV/μs)^2");
        assert_eq!(squared.exp10(), Some(6));

        // V/A is Ω by dimension
        let k_ohm = UnitExpr::from(Unit::new(Prefix::Kilo, BaseUnit::Ohm, 1));
        let v_per_ma =
            UnitExpr::from(BaseUnit::Volt) / (Unit::new(Prefix::Milli, BaseUnit::Ampere, 1));
        assert!(approx_eq!(
            f64,
            v_per_ma.convert_f64(4.7, &k_ohm).unwrap(),
            4.7,
            ulps = 4
        ));
        assert_eq!(
            k_ohm.convert_f64(1.0, &UnitExpr::from(BaseUnit::Volt)),
            None
        );

        assert_eq!(UnitExpr::from(BaseUnit::Unknown("VDC".into())).dim(), None);
        assert_eq!(Scale::Pow10(-3).factor(), 1e-3);
        assert_eq!(Scale::Factor(0.5), Scale::Factor(0.5));
    }

    #[test]
    fn registry() {
        // UNIT-9
        let mut units = Registry::new();
        let rpm = NamedUnit {
            name: "revolutions per minute".into(),
            symbol: "Bd".into(),
            dim: Dim::new(-1, 0, 0, 0, 0, 0, 0),
            scale: Scale::Factor(std::f64::consts::TAU / 60.0),
            aliases: vec!["baud".into()],
        };
        let base = units.register(rpm.clone()).unwrap();
        assert_eq!(units.register(rpm.clone()).unwrap(), base);
        units.alias("Vdc", Unit::base(BaseUnit::Volt)).unwrap();
        units.alias("Ohms", Unit::base(BaseUnit::Ohm)).unwrap();
        units
            .alias("ppb", Unit::new(Prefix::Nano, BaseUnit::Ratio, 1))
            .unwrap();

        let q = units.parse_quantity("2 kBd").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Kilo, base.clone(), 1));
        assert_eq!(q.as_f64().unwrap(), 2000.0);
        assert!(approx_eq!(
            f64,
            q.to_si_f64().unwrap(),
            2000.0 * std::f64::consts::TAU / 60.0,
            ulps = 2
        ));
        let q = units.parse_quantity("60 baud").unwrap();
        assert_eq!(unit_of(&q).base, base);

        let q = units.parse_quantity("12Vdc").unwrap();
        assert_eq!(*unit_of(&q), Unit::base(BaseUnit::Volt));
        let q = units.parse_quantity("3 ppb").unwrap();
        assert_eq!(q.as_f64().unwrap(), 3e-9);
        assert!(units.parse_quantity("3 kppb").is_err());
        let rpm = units
            .register(NamedUnit {
                name: "revolutions per minute".into(),
                symbol: "rpm".into(),
                dim: Dim::new(-1, 0, 0, 0, 0, 0, 0),
                scale: Scale::Factor(std::f64::consts::TAU / 60.0),
                aliases: vec![],
            })
            .unwrap();
        let q = units.parse_quantity("3000 rpm").unwrap();
        assert_eq!(*unit_of(&q), Unit::base(rpm.clone()));
        let q = units.parse_quantity("3 krpm").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Kilo, rpm, 1));
        let q = units.parse_quantity("12 kOhms").unwrap();
        assert_eq!(*unit_of(&q), Unit::new(Prefix::Kilo, BaseUnit::Ohm, 1));
        assert_eq!(units.parse_unit("kVdc").unwrap().to_string(), "kV");

        // without the registry the words stay unknown
        let q = Quantity::parse("12Vdc").unwrap();
        assert_eq!(unit_of(&q).base, BaseUnit::Unknown("Vdc".into()));
        assert!(q.to_si_f64().is_err());

        // SI symbols and taken words can't be redefined
        assert!(matches!(
            units.alias("V", Unit::base(BaseUnit::Ohm)),
            Err(Error::Registry(_))
        ));
        assert!(matches!(
            units.alias("%", Unit::one()),
            Err(Error::Registry(_))
        ));
        assert!(matches!(
            units.alias("baud", Unit::base(BaseUnit::Hertz)),
            Err(Error::Registry(_))
        ));
    }

    #[test]
    fn whole_known_units_win_over_a_prefix_split() {
        // UNIT-12
        for (s, base) in [
            ("1 Pa", BaseUnit::Pascal),
            ("1 cd", BaseUnit::Candela),
            ("1 mol", BaseUnit::Mole),
            ("1 kat", BaseUnit::Katal),
            ("1 rad", BaseUnit::Radian),
            ("1 Gy", BaseUnit::Gray),
            ("1 Hz", BaseUnit::Hertz),
            ("1 T", BaseUnit::Tesla),
            ("1 K", BaseUnit::Kelvin),
            ("1 m", BaseUnit::Meter),
        ] {
            let q = Quantity::parse(s).unwrap();
            assert_eq!(*unit_of(&q), Unit::base(base), "{s}");
        }
        for (s, prefix, base) in [
            ("1 kPa", Prefix::Kilo, BaseUnit::Pascal),
            ("1 mcd", Prefix::Milli, BaseUnit::Candela),
            ("1 mmol", Prefix::Milli, BaseUnit::Mole),
            ("1 dam", Prefix::Deca, BaseUnit::Meter),
            ("1 dm", Prefix::Deci, BaseUnit::Meter),
            ("1 Tm", Prefix::Tera, BaseUnit::Meter),
            ("1 ms", Prefix::Milli, BaseUnit::Second),
            ("1 Kg", Prefix::Kilo, BaseUnit::Gram),
        ] {
            let q = Quantity::parse(s).unwrap();
            assert_eq!(*unit_of(&q), Unit::new(prefix, base, 1), "{s}");
        }
        // unknown words keep the old split
        let q = Quantity::parse("10mVDC").unwrap();
        assert_eq!(
            *unit_of(&q),
            Unit::new(Prefix::Milli, BaseUnit::Unknown("VDC".into()), 1)
        );
        let q = Quantity::parse("10 VDC").unwrap();
        assert_eq!(unit_of(&q).base, BaseUnit::Unknown("VDC".into()));
        // ratios take no further prefix
        assert!(Quantity::parse("1 k%").is_err());
        assert!(Quantity::parse("1 kppm").is_err());
    }

    #[test]
    fn prefixes_before_non_ascii_units() {
        // UNIT-13
        for (s, prefix, base) in [
            ("100 mΩ", Prefix::Milli, BaseUnit::Ohm),
            ("4.7 kΩ", Prefix::Kilo, BaseUnit::Ohm),
            ("4.7 k\u{2126}", Prefix::Kilo, BaseUnit::Ohm),
            ("10 uΩ", Prefix::Micro, BaseUnit::Ohm),
            ("10 \u{b5}Ω", Prefix::Micro, BaseUnit::Ohm),
            ("10 \u{3bc}Ω", Prefix::Micro, BaseUnit::Ohm),
            ("10 \u{b5}F", Prefix::Micro, BaseUnit::Farad),
            ("10 \u{3bc}F", Prefix::Micro, BaseUnit::Farad),
            ("5 m°C", Prefix::Milli, BaseUnit::DegreeCelsius),
            ("25 \u{2103}", Prefix::Unit, BaseUnit::DegreeCelsius),
        ] {
            let q = Quantity::parse(s).unwrap();
            assert_eq!(*unit_of(&q), Unit::new(prefix, base, 1), "{s}");
        }
        // the micro sign alone is a word like `u` alone, not a prefix without unit
        let q = Quantity::parse("10 \u{b5}").unwrap();
        assert_eq!(unit_of(&q).base, BaseUnit::Unknown("\u{b5}".into()));
    }

    #[test]
    fn exponent_without_unit() {
        // NUM-4
        let q = Quantity::parse("1e3").unwrap();
        assert_eq!(q.unit, UnitExpr::one());
        assert_eq!(q.as_f64().unwrap(), 1000.0);
        let q = Quantity::parse("2.5e-3").unwrap();
        assert_eq!(q.base_decimal().unwrap(), "0.0025");
    }

    #[test]
    fn unit_expressions() {
        // EXPR-1, EXPR-2
        let v = || UnitExpr::from(BaseUnit::Volt);
        let us = Unit::new(Prefix::Micro, BaseUnit::Second, 1);
        for s in ["10 V/us", "10V/us", "10*V/us", "10 V / us"] {
            let q = Quantity::parse(s).unwrap();
            assert_eq!(q.unit, v() / us.clone(), "{s}");
            assert_eq!(q.number, "10");
        }
        let q = Quantity::parse("1 m/s²").unwrap();
        assert_eq!(
            q.unit,
            UnitExpr::from(BaseUnit::Meter) / Unit::new(Prefix::Unit, BaseUnit::Second, 2)
        );
        assert_eq!(q.unit.dim(), Some(Dim::new(-2, 1, 0, 0, 0, 0, 0)));

        let q = Quantity::parse("50 ppm/°C").unwrap();
        assert_eq!(
            q.unit,
            UnitExpr::from(Unit::new(Prefix::Micro, BaseUnit::Ratio, 1)) / BaseUnit::DegreeCelsius
        );

        // left associative: a/b*c is (a/b)*c
        let e = Registry::new().parse_unit("kg·m/s^2").unwrap();
        let kg = Unit::new(Prefix::Kilo, BaseUnit::Gram, 1);
        let s2 = Unit::new(Prefix::Unit, BaseUnit::Second, 2);
        assert_eq!(e, UnitExpr::from(kg) * BaseUnit::Meter / s2);
        assert_eq!(e.dim(), BaseUnit::Newton.dim());

        let e = Registry::new().parse_unit("W/(m⋅K)").unwrap();
        assert_eq!(
            e,
            UnitExpr::from(BaseUnit::Watt) / (UnitExpr::from(BaseUnit::Meter) * BaseUnit::Kelvin)
        );
        let e = Registry::new().parse_unit("(m/s)^2").unwrap();
        assert_eq!(
            e,
            (UnitExpr::from(BaseUnit::Meter) / BaseUnit::Second).pow(2)
        );
        let e2 = Registry::new().parse_unit("(m/s)²").unwrap();
        assert_eq!(e, e2);

        // V/A has the dimension of Ω, and converts
        let e = Registry::new().parse_unit("V/mA").unwrap();
        assert_eq!(e.dim(), BaseUnit::Ohm.dim());
        let k_ohm = UnitExpr::from(Unit::new(Prefix::Kilo, BaseUnit::Ohm, 1));
        assert!(approx_eq!(
            f64,
            e.convert_f64(4.7, &k_ohm).unwrap(),
            4.7,
            ulps = 2
        ));

        // Display parses back to the same expression
        for s in [
            "kg·m/s^2",
            "W/(m⋅K)",
            "(m/s)^2",
            "ppm/°C",
            "mV/us",
            "m^-1",
            "(V/A)/(m/s)",
        ] {
            let e = Registry::new().parse_unit(s).unwrap();
            assert_eq!(
                Registry::new().parse_unit(&e.to_string()).unwrap(),
                e,
                "{s} -> {e}"
            );
        }
    }

    #[test]
    fn resistance_values() {
        let v = OhmF32::parse("100").unwrap();
        assert_eq!(v.0, 100.0);

        let v = OhmF32::parse("5k").unwrap();
        assert_eq!(v.0, 5000.0);

        let v = OhmF32::parse("5M").unwrap();
        assert_eq!(v.0, 5_000_000.0);

        let v = OhmF32::parse("5G").unwrap();
        assert_eq!(v.0, 5_000_000_000.0);

        let v = OhmF32::parse("1R").unwrap();
        assert_eq!(v.0, 1.0);

        let v = OhmF32::parse("15.5").unwrap();
        assert_eq!(v.0, 15.5);

        let v = OhmF32::parse("5.55R").unwrap();
        assert_eq!(v.0, 5.55);

        let v = OhmF32::parse("1.0k").unwrap();
        assert_eq!(v.0, 1000.0);

        let v = OhmF32::parse("1k2").unwrap();
        assert_eq!(v.0, 1200.0);

        let v = OhmF32::parse("10 kΩ").unwrap();
        assert_eq!(v.0, 10_000.0);

        let v = OhmF32::parse("5Ω").unwrap();
        assert_eq!(v.0, 5.0);

        let v = OhmF32::parse("1 mΩ").unwrap();
        assert_eq!(v.0, 0.001);

        let v = OhmF32::parse("1μΩ").unwrap();
        assert_eq!(v.0, 0.000001);

        let v = OhmF32::parse(" 0 ").unwrap();
        assert_eq!(v.0, 0.0);

        let v = OhmF32::parse(" 0  R ").unwrap();
        assert_eq!(v.0, 0.0);

        let v = OhmF32::parse("49r").unwrap();
        assert_eq!(v.0, 49.0);

        let v = OhmF32::parse(" 499kR ").unwrap();
        assert_eq!(v.0, 499_000.0);

        let v = OhmF32::parse("12 kOhms").unwrap();
        assert_eq!(v.0, 12000.0);

        let v = OhmF32::parse("2 mOhms").unwrap();
        assert_eq!(v.0, 0.002);

        let v = OhmF32::parse("4.7 k\u{2126}").unwrap();
        assert_eq!(v.0, 4700.0);
    }

    #[test]
    fn resistance_exponents() {
        // RES-2
        assert_eq!(OhmF32::parse("1e3").unwrap().0, 1000.0);
        assert_eq!(OhmF32::parse("1.5e3").unwrap().0, 1500.0);
        assert_eq!(OhmF32::parse("2e-3").unwrap().0, 0.002);
        assert_eq!(OhmF32::parse("2.2e3 Ω").unwrap().0, 2200.0);
        assert_eq!(OhmF32::parse("4k7").unwrap().0, 4700.0);
    }
}
