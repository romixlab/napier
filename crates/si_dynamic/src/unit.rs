//! Units: [Prefix], [BaseUnit], one prefixed unit to a power ([Unit]), and products, quotients and powers of them
//! ([UnitExpr]).

use crate::dim::{Dim, Scale, write_superscript};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// An SI prefix.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Prefix {
    Milli,
    Micro,
    Nano,
    Pico,

    Kilo,
    Mega,
    Giga,

    Unit,

    Deca,
    Hecto,
    Tera,
    Peta,
    Exa,
    Zetta,
    Yotta,
    Ronna,
    Quetta,

    Deci,
    Centi,
    Femto,
    Atto,
    Zepto,
    Yocto,
    Ronto,
    Quecto,
}

impl Prefix {
    /// The power of ten: 3 for kilo, -6 for micro.
    pub fn exp10(&self) -> i8 {
        match self {
            Prefix::Quecto => -30,
            Prefix::Ronto => -27,
            Prefix::Yocto => -24,
            Prefix::Zepto => -21,
            Prefix::Atto => -18,
            Prefix::Femto => -15,
            Prefix::Pico => -12,
            Prefix::Nano => -9,
            Prefix::Micro => -6,
            Prefix::Milli => -3,
            Prefix::Centi => -2,
            Prefix::Deci => -1,
            Prefix::Unit => 0,
            Prefix::Deca => 1,
            Prefix::Hecto => 2,
            Prefix::Kilo => 3,
            Prefix::Mega => 6,
            Prefix::Giga => 9,
            Prefix::Tera => 12,
            Prefix::Peta => 15,
            Prefix::Exa => 18,
            Prefix::Zetta => 21,
            Prefix::Yotta => 24,
            Prefix::Ronna => 27,
            Prefix::Quetta => 30,
        }
    }

    /// The prefix of a power of ten, `None` when there is none (4, -4).
    pub fn from_exp10(exp: i8) -> Option<Prefix> {
        Some(match exp {
            -30 => Prefix::Quecto,
            -27 => Prefix::Ronto,
            -24 => Prefix::Yocto,
            -21 => Prefix::Zepto,
            -18 => Prefix::Atto,
            -15 => Prefix::Femto,
            -12 => Prefix::Pico,
            -9 => Prefix::Nano,
            -6 => Prefix::Micro,
            -3 => Prefix::Milli,
            -2 => Prefix::Centi,
            -1 => Prefix::Deci,
            0 => Prefix::Unit,
            1 => Prefix::Deca,
            2 => Prefix::Hecto,
            3 => Prefix::Kilo,
            6 => Prefix::Mega,
            9 => Prefix::Giga,
            12 => Prefix::Tera,
            15 => Prefix::Peta,
            18 => Prefix::Exa,
            21 => Prefix::Zetta,
            24 => Prefix::Yotta,
            27 => Prefix::Ronna,
            30 => Prefix::Quetta,
            _ => return None,
        })
    }

    pub fn as_f64(&self) -> f64 {
        10f64.powi(self.exp10() as i32)
    }

    pub fn as_f32(&self) -> f32 {
        10f32.powi(self.exp10() as i32)
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Prefix::Milli => "m",
            Prefix::Micro => "μ", // U+03BC
            Prefix::Nano => "n",
            Prefix::Pico => "p",
            Prefix::Kilo => "k",
            Prefix::Mega => "M",
            Prefix::Giga => "G",
            Prefix::Unit => "",
            Prefix::Deca => "da",
            Prefix::Hecto => "h",
            Prefix::Tera => "T",
            Prefix::Peta => "P",
            Prefix::Exa => "E",
            Prefix::Zetta => "Z",
            Prefix::Yotta => "Y",
            Prefix::Ronna => "R",
            Prefix::Quetta => "Q",
            Prefix::Deci => "d",
            Prefix::Centi => "c",
            Prefix::Femto => "f",
            Prefix::Atto => "a",
            Prefix::Zepto => "z",
            Prefix::Yocto => "y",
            Prefix::Ronto => "r",
            Prefix::Quecto => "q",
        }
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// A unit without prefix: the 7 SI base units (with the gram in place of the kilogram, so `kg` is kilo + gram), the
/// 22 derived ones, [Ratio](BaseUnit::Ratio) for dimensionless numbers, units registered at run time
/// ([Named](BaseUnit::Named), see [Registry](crate::Registry)) and words the parser didn't know
/// ([Unknown](BaseUnit::Unknown)).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BaseUnit {
    Second,
    Meter,
    /// 10⁻³ kg: the SI base unit of mass is the kilogram, written as kilo + gram.
    Gram,
    Ampere,
    Kelvin,
    Mole,
    Candela,

    Radian,
    Steradian,
    Hertz,
    Newton,
    Pascal,
    Joule,
    Watt,
    Coulomb,
    Volt,
    Farad,
    Ohm,
    Siemens,
    Weber,
    Tesla,
    Henry,
    /// K − 273.15, an affine scale: converts to kelvin only on its own (not in a product, not to a power).
    DegreeCelsius,
    Lumen,
    Lux,
    Becquerel,
    Gray,
    Sievert,
    Katal,

    /// A dimensionless number. With a prefix it is written `%` (centi), `‰` (milli) and `ppm` (micro), as in ww_si.
    Ratio,
    /// A unit registered at run time ([Registry](crate::Registry)): what it is travels with it, so a stored value
    /// converts without the registry.
    Named {
        /// What it is called: `revolutions per minute`.
        name: String,
        /// As written after a number: `rpm`.
        symbol: String,
        dim: Dim,
        scale: Scale,
    },
    /// A word the parser didn't know (`VDC` in `10mVDC`): no dimension, compares only by the word.
    Unknown(String),
}

/// The SI base and derived units, in the order of [BaseUnit].
pub const KNOWN_UNITS: [BaseUnit; 7 + 22] = [
    BaseUnit::Second,
    BaseUnit::Meter,
    BaseUnit::Gram,
    BaseUnit::Ampere,
    BaseUnit::Kelvin,
    BaseUnit::Mole,
    BaseUnit::Candela,
    BaseUnit::Radian,
    BaseUnit::Steradian,
    BaseUnit::Hertz,
    BaseUnit::Newton,
    BaseUnit::Pascal,
    BaseUnit::Joule,
    BaseUnit::Watt,
    BaseUnit::Coulomb,
    BaseUnit::Volt,
    BaseUnit::Farad,
    BaseUnit::Ohm,
    BaseUnit::Siemens,
    BaseUnit::Weber,
    BaseUnit::Tesla,
    BaseUnit::Henry,
    BaseUnit::DegreeCelsius,
    BaseUnit::Lumen,
    BaseUnit::Lux,
    BaseUnit::Becquerel,
    BaseUnit::Gray,
    BaseUnit::Sievert,
    BaseUnit::Katal,
];

impl BaseUnit {
    /// SI base-unit exponents, `None` for an [Unknown](BaseUnit::Unknown) word.
    pub fn dim(&self) -> Option<Dim> {
        let d = Dim::new;
        Some(match self {
            BaseUnit::Second => d(1, 0, 0, 0, 0, 0, 0),
            BaseUnit::Meter => d(0, 1, 0, 0, 0, 0, 0),
            BaseUnit::Gram => d(0, 0, 1, 0, 0, 0, 0),
            BaseUnit::Ampere => d(0, 0, 0, 1, 0, 0, 0),
            BaseUnit::Kelvin => d(0, 0, 0, 0, 1, 0, 0),
            BaseUnit::Mole => d(0, 0, 0, 0, 0, 1, 0),
            BaseUnit::Candela => d(0, 0, 0, 0, 0, 0, 1),
            BaseUnit::Radian | BaseUnit::Steradian | BaseUnit::Ratio => Dim::ONE,
            BaseUnit::Hertz | BaseUnit::Becquerel => d(-1, 0, 0, 0, 0, 0, 0),
            BaseUnit::Newton => d(-2, 1, 1, 0, 0, 0, 0),
            BaseUnit::Pascal => d(-2, -1, 1, 0, 0, 0, 0),
            BaseUnit::Joule => d(-2, 2, 1, 0, 0, 0, 0),
            BaseUnit::Watt => d(-3, 2, 1, 0, 0, 0, 0),
            BaseUnit::Coulomb => d(1, 0, 0, 1, 0, 0, 0),
            BaseUnit::Volt => d(-3, 2, 1, -1, 0, 0, 0),
            BaseUnit::Farad => d(4, -2, -1, 2, 0, 0, 0),
            BaseUnit::Ohm => d(-3, 2, 1, -2, 0, 0, 0),
            BaseUnit::Siemens => d(3, -2, -1, 2, 0, 0, 0),
            BaseUnit::Weber => d(-2, 2, 1, -1, 0, 0, 0),
            BaseUnit::Tesla => d(-2, 0, 1, -1, 0, 0, 0),
            BaseUnit::Henry => d(-2, 2, 1, -2, 0, 0, 0),
            BaseUnit::DegreeCelsius => d(0, 0, 0, 0, 1, 0, 0),
            BaseUnit::Lumen => d(0, 0, 0, 0, 0, 0, 1),
            BaseUnit::Lux => d(0, -2, 0, 0, 0, 0, 1),
            BaseUnit::Gray | BaseUnit::Sievert => d(-2, 2, 0, 0, 0, 0, 0),
            BaseUnit::Katal => d(-1, 0, 0, 0, 0, 1, 0),
            BaseUnit::Named { dim, .. } => *dim,
            BaseUnit::Unknown(_) => return None,
        })
    }

    /// How a value in this unit becomes coherent SI, `None` for an [Unknown](BaseUnit::Unknown) word.
    pub fn scale(&self) -> Option<Scale> {
        Some(match self {
            BaseUnit::Gram => Scale::Pow10(-3),
            BaseUnit::DegreeCelsius => Scale::Affine {
                factor: 1.0,
                offset: 273.15,
            },
            BaseUnit::Named { scale, .. } => *scale,
            BaseUnit::Unknown(_) => return None,
            _ => Scale::Coherent,
        })
    }

    pub fn name(&self) -> &str {
        match self {
            BaseUnit::Second => "Second",
            BaseUnit::Meter => "Meter",
            BaseUnit::Gram => "Gram",
            BaseUnit::Ampere => "Ampere",
            BaseUnit::Kelvin => "Kelvin",
            BaseUnit::Mole => "Mole",
            BaseUnit::Candela => "Candela",
            BaseUnit::Radian => "Radian",
            BaseUnit::Steradian => "Steradian",
            BaseUnit::Hertz => "Hertz",
            BaseUnit::Newton => "Newton",
            BaseUnit::Pascal => "Pascal",
            BaseUnit::Joule => "Joule",
            BaseUnit::Watt => "Watt",
            BaseUnit::Coulomb => "Coulomb",
            BaseUnit::Volt => "Volt",
            BaseUnit::Farad => "Farad",
            BaseUnit::Ohm => "Ohm",
            BaseUnit::Siemens => "Siemens",
            BaseUnit::Weber => "Weber",
            BaseUnit::Tesla => "Tesla",
            BaseUnit::Henry => "Henry",
            BaseUnit::DegreeCelsius => "Degree Celsius",
            BaseUnit::Lumen => "Lumen",
            BaseUnit::Lux => "Lux",
            BaseUnit::Becquerel => "Becquerel",
            BaseUnit::Gray => "Gray",
            BaseUnit::Sievert => "Sievert",
            BaseUnit::Katal => "Katal",
            BaseUnit::Ratio => "Ratio",
            BaseUnit::Named { name, .. } => name,
            BaseUnit::Unknown(word) => word,
        }
    }

    /// As written after a number (and a prefix): `V`, `g`, `Ω`; empty for [Ratio](BaseUnit::Ratio).
    pub fn symbol(&self) -> &str {
        match self {
            BaseUnit::Second => "s",
            BaseUnit::Meter => "m",
            BaseUnit::Gram => "g",
            BaseUnit::Ampere => "A",
            BaseUnit::Kelvin => "K",
            BaseUnit::Mole => "mol",
            BaseUnit::Candela => "cd",
            BaseUnit::Radian => "rad",
            BaseUnit::Steradian => "sr",
            BaseUnit::Hertz => "Hz",
            BaseUnit::Newton => "N",
            BaseUnit::Pascal => "Pa",
            BaseUnit::Joule => "J",
            BaseUnit::Watt => "W",
            BaseUnit::Coulomb => "C",
            BaseUnit::Volt => "V",
            BaseUnit::Farad => "F",
            BaseUnit::Ohm => "Ω",
            BaseUnit::Siemens => "S",
            BaseUnit::Weber => "Wb",
            BaseUnit::Tesla => "T",
            BaseUnit::Henry => "H",
            BaseUnit::DegreeCelsius => "°C",
            BaseUnit::Lumen => "lm",
            BaseUnit::Lux => "lx",
            BaseUnit::Becquerel => "Bq",
            BaseUnit::Gray => "Gy",
            BaseUnit::Sievert => "Sv",
            BaseUnit::Katal => "kat",
            BaseUnit::Ratio => "",
            BaseUnit::Named { symbol, .. } => symbol,
            BaseUnit::Unknown(word) => word,
        }
    }

    /// The SI unit written as `symbol`, `None` for anything else.
    pub fn from_symbol(symbol: &str) -> Option<BaseUnit> {
        KNOWN_UNITS.iter().find(|u| u.symbol() == symbol).cloned()
    }
}

impl Display for BaseUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}

/// One unit with a prefix, to an integer power: `km²` is `Unit { prefix: Kilo, base: Meter, exp: 2 }`, and the prefix
/// is raised with it (1 km² = 10⁶ m²).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Unit {
    pub prefix: Prefix,
    pub base: BaseUnit,
    pub exp: i8,
}

impl Unit {
    /// A dimensionless number: no prefix, [BaseUnit::Ratio].
    pub fn one() -> Self {
        Unit::base(BaseUnit::Ratio)
    }

    /// `base` without prefix, to the power of one.
    pub fn base(base: BaseUnit) -> Self {
        Unit {
            prefix: Prefix::Unit,
            base,
            exp: 1,
        }
    }

    pub fn new(prefix: Prefix, base: BaseUnit, exp: i8) -> Self {
        Unit { prefix, base, exp }
    }

    /// SI base-unit exponents, `None` for an unknown word or when an exponent overflows.
    pub fn dim(&self) -> Option<Dim> {
        self.base.dim()?.checked_pow(self.exp)
    }

    /// The power of ten of the prefix, raised to the exponent: 6 for km².
    pub fn prefix_exp10(&self) -> i32 {
        self.prefix.exp10() as i32 * self.exp as i32
    }

    /// The power of ten to coherent SI, prefix and scale raised to the exponent: 6 for km², 0 for kg, -6 for ppm;
    /// `None` when the scale is not a power of ten (°C, a `Factor`) or the word is unknown.
    pub fn exp10(&self) -> Option<i32> {
        Some((self.prefix.exp10() as i32 + self.base.scale()?.exp10()?) * self.exp as i32)
    }

    /// The factor to coherent SI, prefix and scale raised to the exponent (the offset left out).
    pub fn factor(&self) -> Option<f64> {
        let f = self.prefix.as_f64() * self.base.scale()?.factor();
        Some(f.powi(self.exp as i32))
    }

    /// The offset to coherent SI: only a unit with an `Affine` scale (°C) to the power of one has one.
    pub fn offset(&self) -> f64 {
        match self.base.scale() {
            Some(s) if self.exp == 1 => s.offset(),
            _ => 0.0,
        }
    }

    /// The same unit without its prefix.
    pub fn without_prefix(&self) -> Unit {
        Unit {
            prefix: Prefix::Unit,
            ..self.clone()
        }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.base == BaseUnit::Ratio {
            match self.prefix {
                Prefix::Unit => {}
                Prefix::Centi => f.write_str("%")?,
                Prefix::Milli => f.write_str("‰")?,
                Prefix::Micro => f.write_str("ppm")?,
                p => {
                    f.write_str("×10")?;
                    write_superscript(f, p.exp10())?;
                }
            }
        } else {
            write!(f, "{}{}", self.prefix, self.base)?;
        }
        if self.exp != 1 {
            // TODO(UNIT-8): superscript
            write!(f, "^{}", self.exp)?;
        }
        Ok(())
    }
}

/// A unit as a tree: one [Unit], or products, quotients and powers of units (`ppm/°C` is
/// `Div(Unit(ppm), Unit(°C))`), the same shape as ww_si's `UnitExpr`. Prefixes live in the [Unit]s, so [factor]
/// (Self::factor) and [exp10](Self::exp10) include them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UnitExpr {
    Unit(Unit),
    Mul(Box<UnitExpr>, Box<UnitExpr>),
    Div(Box<UnitExpr>, Box<UnitExpr>),
    /// A compound unit to an integer power: `(m/s)²`. One unit to a power is a [Unit] with its `exp`.
    Pow(Box<UnitExpr>, i8),
}

impl From<Unit> for UnitExpr {
    fn from(u: Unit) -> Self {
        UnitExpr::Unit(u)
    }
}

impl From<BaseUnit> for UnitExpr {
    fn from(b: BaseUnit) -> Self {
        UnitExpr::Unit(Unit::base(b))
    }
}

/// `a * b` is `Mul(a, b)`, not reduced.
impl<R: Into<UnitExpr>> std::ops::Mul<R> for UnitExpr {
    type Output = UnitExpr;

    fn mul(self, rhs: R) -> UnitExpr {
        UnitExpr::Mul(Box::new(self), Box::new(rhs.into()))
    }
}

/// `a / b` is `Div(a, b)`, not reduced.
impl<R: Into<UnitExpr>> std::ops::Div<R> for UnitExpr {
    type Output = UnitExpr;

    fn div(self, rhs: R) -> UnitExpr {
        UnitExpr::Div(Box::new(self), Box::new(rhs.into()))
    }
}

impl UnitExpr {
    /// A dimensionless number, [Unit::one].
    pub fn one() -> Self {
        UnitExpr::Unit(Unit::one())
    }

    pub fn pow(self, n: i8) -> Self {
        UnitExpr::Pow(Box::new(self), n)
    }

    /// The single unit, if this is one.
    pub fn as_unit(&self) -> Option<&Unit> {
        match self {
            UnitExpr::Unit(u) => Some(u),
            _ => None,
        }
    }

    /// SI base-unit exponents of the whole expression; `None` with an unknown word or when an exponent overflows.
    pub fn dim(&self) -> Option<Dim> {
        match self {
            UnitExpr::Unit(u) => u.dim(),
            UnitExpr::Mul(a, b) => a.dim()?.checked_mul(b.dim()?),
            UnitExpr::Div(a, b) => a.dim()?.checked_div(b.dim()?),
            UnitExpr::Pow(a, n) => a.dim()?.checked_pow(*n),
        }
    }

    /// The factor to coherent SI with every prefix (offsets left out: a product with °C is not affine); `None` with
    /// an unknown word.
    pub fn factor(&self) -> Option<f64> {
        Some(match self {
            UnitExpr::Unit(u) => u.factor()?,
            UnitExpr::Mul(a, b) => a.factor()? * b.factor()?,
            UnitExpr::Div(a, b) => a.factor()? / b.factor()?,
            UnitExpr::Pow(a, n) => a.factor()?.powi(*n as i32),
        })
    }

    /// The offset to coherent SI: only a single unit with an `Affine` scale (°C) has one.
    pub fn offset(&self) -> f64 {
        match self {
            UnitExpr::Unit(u) => u.offset(),
            _ => 0.0,
        }
    }

    /// Whether a unit in the expression has an `Affine` scale (°C).
    pub fn has_offset(&self) -> bool {
        match self {
            UnitExpr::Unit(u) => matches!(u.base.scale(), Some(Scale::Affine { .. })),
            UnitExpr::Mul(a, b) | UnitExpr::Div(a, b) => a.has_offset() || b.has_offset(),
            UnitExpr::Pow(a, _) => a.has_offset(),
        }
    }

    /// The power of ten to coherent SI with every prefix and scale (`mV/µs` is 3, `kg` is 0, `ppm/°C` is -6 ...
    /// without the °C offset); `None` when a scale is not a power of ten or a word is unknown. For exact conversions.
    pub fn exp10(&self) -> Option<i32> {
        match self {
            UnitExpr::Unit(u) => u.exp10(),
            UnitExpr::Mul(a, b) => Some(a.exp10()? + b.exp10()?),
            UnitExpr::Div(a, b) => Some(a.exp10()? - b.exp10()?),
            UnitExpr::Pow(a, n) => Some(a.exp10()? * *n as i32),
        }
    }

    /// The power of ten of the prefixes alone: the value in this unit times `10^prefix_exp10` is the value in the
    /// same unit without prefixes ([without_prefixes](Self::without_prefixes)). 6 for km², 3 for kg, 3 for mV/µs.
    pub fn prefix_exp10(&self) -> i32 {
        match self {
            UnitExpr::Unit(u) => u.prefix_exp10(),
            UnitExpr::Mul(a, b) => a.prefix_exp10() + b.prefix_exp10(),
            UnitExpr::Div(a, b) => a.prefix_exp10() - b.prefix_exp10(),
            UnitExpr::Pow(a, n) => a.prefix_exp10() * *n as i32,
        }
    }

    /// The same expression with every prefix removed: `kΩ` → `Ω`, `kg` → `g`, `mV/µs` → `V/s`, `ppm` → ratio.
    pub fn without_prefixes(&self) -> UnitExpr {
        match self {
            UnitExpr::Unit(u) => UnitExpr::Unit(u.without_prefix()),
            UnitExpr::Mul(a, b) => a.without_prefixes() * b.without_prefixes(),
            UnitExpr::Div(a, b) => a.without_prefixes() / b.without_prefixes(),
            UnitExpr::Pow(a, n) => a.without_prefixes().pow(*n),
        }
    }

    /// Whether both are the same expression up to prefixes (`kΩ` and `mΩ`, `mV/µs` and `V/s`), so a value converts
    /// by a power of ten alone.
    pub fn same_but_prefixes(&self, other: &UnitExpr) -> bool {
        self.without_prefixes() == other.without_prefixes()
    }

    /// `value` in this unit, in coherent SI: `value · factor + offset`; `None` with an unknown word or with °C
    /// inside a compound unit.
    pub fn to_si_f64(&self, value: f64) -> Option<f64> {
        if self.has_offset() && self.as_unit().is_none_or(|u| u.exp != 1) {
            return None;
        }
        Some(value * self.factor()? + self.offset())
    }

    /// `value` in this unit, in `to`, through coherent SI (°C → K takes the offset); `None` when the dimensions
    /// differ or are unknown, or with °C inside a compound unit.
    pub fn convert_f64(&self, value: f64, to: &UnitExpr) -> Option<f64> {
        if self.dim()? != to.dim()? {
            return None;
        }
        let si = self.to_si_f64(value)?;
        // the inverse of `to_si_f64` for `to`
        to.to_si_f64(0.0)?;
        Some((si - to.offset()) / to.factor()?)
    }
}

/// The symbol: `kΩ`, `ppm/°C`, `kg·m/s^2`; a compound operand in parentheses.
impl Display for UnitExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn operand(e: &UnitExpr, f: &mut Formatter<'_>) -> fmt::Result {
            match e {
                UnitExpr::Unit(_) | UnitExpr::Pow(..) => write!(f, "{e}"),
                _ => write!(f, "({e})"),
            }
        }
        match self {
            UnitExpr::Unit(u) => write!(f, "{u}"),
            UnitExpr::Mul(a, b) => {
                operand(a, f)?;
                f.write_str("·")?;
                operand(b, f)
            }
            UnitExpr::Div(a, b) => {
                operand(a, f)?;
                f.write_str("/")?;
                operand(b, f)
            }
            UnitExpr::Pow(a, n) => {
                match **a {
                    UnitExpr::Unit(_) => write!(f, "({a})")?,
                    _ => operand(a, f)?,
                }
                write!(f, "^{n}")
            }
        }
    }
}
