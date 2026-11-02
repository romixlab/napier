//! SI dimension of a unit and how its values become coherent SI, the same model as ww_si (wire_weaver's `ww_si`):
//! the same exponent order in [Dim] and the same [Scale] variants, so a bridge between the two is mechanical.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

/// SI base-unit exponents of a unit: s, m, kg, A, K, mol, cd (ww_si's order). Volt is kg·m²·s⁻³·A⁻¹:
/// `Dim::new(-3, 2, 1, -1, 0, 0, 0)`.
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Dim {
    pub s: i8,
    pub m: i8,
    pub kg: i8,
    pub a: i8,
    pub k: i8,
    pub mol: i8,
    pub cd: i8,
}

impl Dim {
    /// Dimensionless: a ratio, an angle in radians.
    pub const ONE: Dim = Dim::new(0, 0, 0, 0, 0, 0, 0);

    /// From the exponents of s, m, kg, A, K, mol, cd.
    pub const fn new(s: i8, m: i8, kg: i8, a: i8, k: i8, mol: i8, cd: i8) -> Dim {
        Dim {
            s,
            m,
            kg,
            a,
            k,
            mol,
            cd,
        }
    }

    /// The exponents of s, m, kg, A, K, mol, cd.
    pub const fn exps(&self) -> [i8; 7] {
        [self.s, self.m, self.kg, self.a, self.k, self.mol, self.cd]
    }

    fn from_exps(e: [i8; 7]) -> Dim {
        Dim::new(e[0], e[1], e[2], e[3], e[4], e[5], e[6])
    }

    fn combine(self, o: Dim, f: impl Fn(i8, i8) -> Option<i8>) -> Option<Dim> {
        let (a, b) = (self.exps(), o.exps());
        let mut out = [0i8; 7];
        for i in 0..7 {
            out[i] = f(a[i], b[i])?;
        }
        Some(Dim::from_exps(out))
    }

    /// The dimension of a product: exponents added. `None` when one overflows `i8`.
    pub fn checked_mul(self, o: Dim) -> Option<Dim> {
        self.combine(o, i8::checked_add)
    }

    /// The dimension of a quotient: exponents subtracted. `None` when one overflows `i8`.
    pub fn checked_div(self, o: Dim) -> Option<Dim> {
        self.combine(o, i8::checked_sub)
    }

    /// The dimension of a power: exponents multiplied by `n`. `None` when one overflows `i8`.
    pub fn checked_pow(self, n: i8) -> Option<Dim> {
        self.combine(Dim::ONE, |a, _| a.checked_mul(n))
    }

    pub fn is_one(&self) -> bool {
        *self == Dim::ONE
    }
}

/// Written the SI way, base units in the order kg, m, s, A, K, mol, cd: `kg·m²·s⁻³·A⁻¹`, `1` when dimensionless.
impl Display for Dim {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let parts = [
            ("kg", self.kg),
            ("m", self.m),
            ("s", self.s),
            ("A", self.a),
            ("K", self.k),
            ("mol", self.mol),
            ("cd", self.cd),
        ];
        let mut first = true;
        for (symbol, exp) in parts {
            if exp == 0 {
                continue;
            }
            if !first {
                f.write_str("·")?;
            }
            first = false;
            f.write_str(symbol)?;
            if exp != 1 {
                write_superscript(f, exp)?;
            }
        }
        if first {
            f.write_str("1")?;
        }
        Ok(())
    }
}

pub(crate) fn write_superscript(f: &mut Formatter<'_>, exp: i8) -> fmt::Result {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    if exp < 0 {
        f.write_str("⁻")?;
    }
    for d in exp.unsigned_abs().to_string().bytes() {
        write!(f, "{}", DIGITS[(d - b'0') as usize])?;
    }
    Ok(())
}

/// How a value in a unit (without prefix) becomes coherent SI: `si = value · factor + offset`.
///
/// `Eq`, `Ord` and `Hash` compare the `f64`s bit by bit, so a unit can be a key and sit in an ordered value.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Scale {
    /// The unit is coherent SI: V, A, N, rad.
    Coherent,
    /// A power of ten: g is 10⁻³ kg, L is 10⁻³ m³.
    Pow10(i8),
    /// Any factor: ° is π/180 rad, rpm is 2π/60 rad/s.
    Factor(f64),
    /// A factor and an offset: °C is `Affine { factor: 1.0, offset: 273.15 }`.
    Affine { factor: f64, offset: f64 },
}

impl Scale {
    /// The factor, without the offset of an `Affine` scale.
    pub fn factor(&self) -> f64 {
        match *self {
            Scale::Coherent => 1.0,
            Scale::Pow10(exp) => 10f64.powi(exp as i32),
            Scale::Factor(factor) => factor,
            Scale::Affine { factor, .. } => factor,
        }
    }

    /// The offset of an `Affine` scale, `0.0` for the others.
    pub fn offset(&self) -> f64 {
        match *self {
            Scale::Affine { offset, .. } => offset,
            _ => 0.0,
        }
    }

    /// The power of ten of a `Coherent` (0) or `Pow10` scale, for exact conversions; `None` for the others.
    pub fn exp10(&self) -> Option<i32> {
        match *self {
            Scale::Coherent => Some(0),
            Scale::Pow10(e) => Some(e as i32),
            _ => None,
        }
    }

    fn key(&self) -> (u8, i8, u64, u64) {
        match *self {
            Scale::Coherent => (0, 0, 0, 0),
            Scale::Pow10(e) => (1, e, 0, 0),
            Scale::Factor(f) => (2, 0, f.to_bits(), 0),
            Scale::Affine { factor, offset } => (3, 0, factor.to_bits(), offset.to_bits()),
        }
    }
}

impl PartialEq for Scale {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for Scale {}

impl Hash for Scale {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}

impl PartialOrd for Scale {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Scale {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}
