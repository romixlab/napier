# si_dynamic

Parse SI quantities and units from text at runtime.

```rust
use si_dynamic::{BaseUnit, OhmF32, Prefix, Quantity, Unit};

let q = Quantity::parse("4.7 uF")?;
assert_eq!(q.unit.as_unit(), Some(&Unit::new(Prefix::Micro, BaseUnit::Farad, 1)));
assert_eq!(q.base_decimal()?, "0.0000047");

let q = Quantity::parse("10 V/us")?; // a unit expression
assert_eq!(q.unit.to_string(), "V/μs");

let r = OhmF32::parse("4k7")?; // 4700 Ω
assert_eq!(r.0, 4700.0);
# Ok::<(), si_dynamic::Error>(())
```

All SI base and derived units and prefixes, exponents (`m^2`, `m²`), unit expressions (`m/s²`, `ppm/°C`,
`W/(m·K)`), `%`, `‰`, `ppm`, unknown units kept by name (`10mVDC`), and the resistance shorthand used on schematics
and BOMs (`4k7`, `499kR`, `12 kOhms`). Each unit has its dimension and scale to coherent SI (the model follows
wire_weaver's ww_si); units beyond SI (`rpm`, `°F`) are registered at runtime in a `Registry`.

## Serde stability

`Prefix`, `Unit` and `UnitExpr` (with `BaseUnit`, `Dim` and `Scale` inside) are stored by users of this crate, so
their serde shape is part of the API: it changes only with a new major version (0.x: a new minor version), and the
CHANGELOG says how. [tests/golden/units.json](tests/golden/units.json) holds the shape and `tests/golden.rs` checks
it both ways.

What works, known bugs and plans: [FEATURES.md](FEATURES.md). Changes: [CHANGELOG.md](CHANGELOG.md).

License: MIT.
