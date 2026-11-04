//! Units known at run time (UNIT-9): named units with their dimension and scale, and other spellings of units.
//! Nothing about a particular non-SI unit is built in; a caller (a parts database, a test bench from its
//! config) registers what it needs and parses through its [Registry].

use crate::dim::{Dim, Scale};
use crate::unit::{BaseUnit, Prefix, Unit, UnitExpr};
use crate::{Error, Quantity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A unit to register: `rpm`, revolutions per minute, `Dim::new(-1, ..)`, `Scale::Factor(2π / 60)`, aliases `RPM`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedUnit {
    pub name: String,
    pub symbol: String,
    pub dim: Dim,
    pub scale: Scale,
    /// Other spellings of the same unit.
    #[serde(default)]
    pub aliases: Vec<String>,
}

impl NamedUnit {
    /// The [BaseUnit] values of this unit carry.
    pub fn base(&self) -> BaseUnit {
        BaseUnit::Named {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            dim: self.dim,
            scale: self.scale,
        }
    }
}

/// The words a parser knows beyond the SI symbols and `%`, `‰`, `ppm`: named units and aliases.
///
/// ```
/// use si_dynamic::{BaseUnit, Dim, NamedUnit, Prefix, Registry, Scale, Unit};
///
/// let mut units = Registry::new();
/// units
///     .register(NamedUnit {
///         name: "baud".into(),
///         symbol: "Bd".into(),
///         dim: Dim::new(-1, 0, 0, 0, 0, 0, 0),
///         scale: Scale::Coherent,
///         aliases: vec!["baud".into()],
///     })
///     .unwrap();
/// units.alias("Vdc", Unit::base(BaseUnit::Volt)).unwrap();
///
/// let q = units.parse_quantity("115.2 kBd").unwrap();
/// assert_eq!(q.as_f64().unwrap(), 115_200.0);
/// let q = units.parse_quantity("12 Vdc").unwrap();
/// assert_eq!(q.unit.as_unit().unwrap().base, BaseUnit::Volt);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Registry {
    words: HashMap<String, Unit>,
}

impl Registry {
    /// Only the built-in SI symbols and `%`, `‰`, `ppm`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a named unit under its symbol and aliases. Fails when one of them is an SI symbol or already
    /// stands for another unit; registering the same unit again is fine.
    pub fn register(&mut self, unit: NamedUnit) -> Result<BaseUnit, Error> {
        let base = unit.base();
        for word in std::iter::once(&unit.symbol).chain(&unit.aliases) {
            self.alias(word, Unit::base(base.clone()))?;
        }
        Ok(base)
    }

    /// `word` stands for `unit`: `Ohms` for Ω, `Vdc` for V, `ppb` for nano + ratio. Fails like
    /// [register](Self::register).
    pub fn alias(&mut self, word: &str, unit: Unit) -> Result<(), Error> {
        if word.is_empty() {
            return Err(Error::Registry("empty unit word".into()));
        }
        if let Some(builtin) = builtin(word) {
            return Err(Error::Registry(format!(
                "{word:?} is the built-in unit {builtin}"
            )));
        }
        match self.words.get(word) {
            Some(known) if *known != unit => Err(Error::Registry(format!(
                "{word:?} is already registered as {known}"
            ))),
            _ => {
                self.words.insert(word.to_string(), unit);
                Ok(())
            }
        }
    }

    /// The unit a word stands for: an SI symbol, `%`, `‰`, `ppm`, or a registered word.
    pub fn lookup(&self, word: &str) -> Option<Unit> {
        builtin(word).or_else(|| self.words.get(word).cloned())
    }

    /// A number with an optional unit, `4.7 kΩ`, with the words of this registry.
    pub fn parse_quantity(&self, s: &str) -> Result<Quantity, Error> {
        crate::parse::quantity(s, self)
    }

    /// A unit alone, `kΩ`, with the words of this registry.
    pub fn parse_unit(&self, s: &str) -> Result<UnitExpr, Error> {
        crate::parse::unit_expr(s, self)
    }
}

/// The built-in words.
fn builtin(word: &str) -> Option<Unit> {
    let ratio = |prefix| Unit::new(prefix, BaseUnit::Ratio, 1);
    match word {
        "%" => Some(ratio(Prefix::Centi)),
        "‰" => Some(ratio(Prefix::Milli)),
        "ppm" => Some(ratio(Prefix::Micro)),
        _ => BaseUnit::from_symbol(word).map(Unit::base),
    }
}
