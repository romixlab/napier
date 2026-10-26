//! The serde shape of [Prefix], [Unit] and [UnitExpr] is stored data (rvariant values in mx3's database): it may only
//! change on purpose, with a new major version. `UPDATE_GOLDEN=1 cargo test --test golden` rewrites the file after
//! such a change.

use serde::{Deserialize, Serialize};
use si_dynamic::{BaseUnit, Dim, Prefix, Scale, Unit, UnitExpr};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Golden {
    prefixes: Vec<Prefix>,
    units: Vec<Unit>,
    unit_exprs: Vec<UnitExpr>,
}

fn golden() -> Golden {
    let rpm = BaseUnit::Named {
        name: "revolutions per minute".into(),
        symbol: "rpm".into(),
        dim: Dim::new(-1, 0, 0, 0, 0, 0, 0),
        scale: Scale::Factor(0.10471975511965977),
    };
    let degf = BaseUnit::Named {
        name: "degree Fahrenheit".into(),
        symbol: "°F".into(),
        dim: Dim::new(0, 0, 0, 0, 1, 0, 0),
        scale: Scale::Affine {
            factor: 5.0 / 9.0,
            offset: 255.37222222222223,
        },
    };
    let angstrom = BaseUnit::Named {
        name: "ångström".into(),
        symbol: "Å".into(),
        dim: Dim::new(0, 1, 0, 0, 0, 0, 0),
        scale: Scale::Pow10(-10),
    };
    let baud = BaseUnit::Named {
        name: "baud".into(),
        symbol: "Bd".into(),
        dim: Dim::new(-1, 0, 0, 0, 0, 0, 0),
        scale: Scale::Coherent,
    };
    let ppm = Unit::new(Prefix::Micro, BaseUnit::Ratio, 1);
    Golden {
        prefixes: vec![
            Prefix::Unit,
            Prefix::Kilo,
            Prefix::Micro,
            Prefix::Deca,
            Prefix::Quecto,
        ],
        units: vec![
            Unit::base(BaseUnit::Volt),
            Unit::new(Prefix::Kilo, BaseUnit::Ohm, 1),
            Unit::new(Prefix::Kilo, BaseUnit::Meter, 2),
            Unit::new(Prefix::Unit, BaseUnit::Second, -1),
            Unit::new(Prefix::Kilo, BaseUnit::Gram, 1),
            Unit::base(BaseUnit::DegreeCelsius),
            Unit::one(),
            Unit::new(Prefix::Centi, BaseUnit::Ratio, 1),
            ppm.clone(),
            Unit::base(rpm),
            Unit::base(degf),
            Unit::base(angstrom),
            Unit::new(Prefix::Kilo, baud, 1),
            Unit::new(Prefix::Milli, BaseUnit::Unknown("VDC".into()), 1),
        ],
        unit_exprs: vec![
            UnitExpr::from(Unit::new(Prefix::Micro, BaseUnit::Farad, 1)),
            UnitExpr::from(ppm) / BaseUnit::DegreeCelsius,
            UnitExpr::from(Unit::new(Prefix::Kilo, BaseUnit::Gram, 1)) * BaseUnit::Meter
                / Unit::new(Prefix::Unit, BaseUnit::Second, 2),
            (UnitExpr::from(BaseUnit::Meter) / BaseUnit::Second).pow(2),
        ],
    }
}

#[test]
fn golden_json() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/units.json");
    let json = serde_json::to_string_pretty(&golden()).unwrap() + "\n";
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(path, &json).unwrap();
    }
    let stored = std::fs::read_to_string(path).unwrap();
    assert_eq!(
        json, stored,
        "serde shape changed: see the note at the top of tests/golden.rs"
    );
    let back: Golden = serde_json::from_str(&stored).unwrap();
    assert_eq!(back, golden());
}
