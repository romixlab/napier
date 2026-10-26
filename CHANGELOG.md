# Changelog

All notable changes to si_dynamic. The format follows [Keep a Changelog](https://keepachangelog.com/), versions
follow [Semantic Versioning](https://semver.org/). Feature IDs refer to [FEATURES.md](FEATURES.md).

## [Unreleased]

### Added

- Unit model after ww_si: `Dim`, `Scale` (`Coherent`, `Pow10`, `Factor`, `Affine`), compound `UnitExpr` (`Unit`,
  `Mul`, `Div`, `Pow`) with `dim()`, `factor()`, `offset()`, `exp10()`, `convert_f64()` (UNIT-10).
- `BaseUnit::Ratio`; `%`, `‰`, `ppm` parse as centi, milli, micro + ratio (UNIT-11).
- `Registry`: named units with dimension, scale and aliases registered at run time, aliases for other spellings
  (UNIT-9).
- `Quantity::value_f64`, `as_f64`, `to_si_f64`, exact `base_decimal` and `base_i64` (NUM-5).
- `Prefix::exp10`, `from_exp10`, `as_f64`; `Error::Registry`; `std::error::Error` for `Error`.
- Unit expressions: `10 V/us`, `1 m/s²`, `ppm/°C`, `W/(m·K)`, `(m/s)^2` parse into `UnitExpr`; `·` (U+00B7)
  multiplies besides `*` and `⋅` (EXPR-1, EXPR-2).
- `Ω` as U+2126 and `℃` (U+2103) are read as `Ω` and `°C`; `m°C` is milli + °C (UNIT-13).

### Changed

- **Breaking:** `BaseUnit::Kilogram` is now `BaseUnit::Gram` (`Scale::Pow10(-3)`); `10 kg` parses to kilo + gram,
  `10 g` to gram (was kilogram, so `10 g` now means 10 g) (UNIT-6).
- **Breaking:** `SIExp` is replaced by `Dim` (order s, m, kg, A, K, mol, cd); `BaseUnit::exp()` is `dim()` and
  returns `Option` (UNIT-10).
- **Breaking:** unknown words are `BaseUnit::Unknown(word)` with no dimension (was `Named` with a zero dimension);
  `BaseUnit::Named` is a registered unit with `name`, `symbol`, `dim`, `scale` (UNIT-4, UNIT-9).
- **Breaking:** `BaseUnit::Unitless` and `Unit::unit()` are `BaseUnit::Ratio` and `Unit::one()` (UNIT-11).
- **Breaking:** `Quantity::unit` is a `UnitExpr`; `as_f32` returns `Result<f32, Error>` and raises the prefix to the
  exponent, so `1 km^2` gives 1e6 (was 1000) (NUM-3).
- **Breaking:** `°C` has an affine scale: `25 °C` is 298.15 K in coherent SI (UNIT-7).
- **Breaking:** `Error` is `#[non_exhaustive]`, `Error::Pest` is boxed.
- A whole known unit wins over a prefix split: registered words like `ppb` and `rpm` are the unit (`ppb` was pico +
  `pb`); `k%` and `kppm` are errors (UNIT-12).

### Removed

- **Breaking:** the `SIExpr` stub (it printed the parse tree and returned nothing); `UnitExpr` takes its place
  (EXPR-1).

### Fixed

- Superscript exponents (`m²`, `s⁻¹`) failed to parse (UNIT-3).
- `100 mΩ` and other prefixes before `Ω` failed to parse; `10 µ` failed while `10 u` parsed (UNIT-13).
- `1e3` without unit failed to parse (NUM-4).
- `OhmF32::parse` failed on exponents: `1e3`, `1.5e3`, `2e-3` (RES-2).

## [0.2.0] - 2026-10-07

### Added

- AGENTS.md, FEATURES.md, README.md and this changelog.

## [0.1.0] - 2026-09-07

First release on crates.io. Reconstructed from git history.

### Added

- Pest grammar for SI base and derived units, all prefixes, exponents and superscripts (UNIT-1, UNIT-2, UNIT-3).
- `Quantity::parse`: number plus unit, `as_f32()` (NUM-1); unknown units as `BaseUnit::Named` (UNIT-4);
  `KNOWN_UNITS`.
- `OhmF32::parse` for resistance values like `4k7`, `499kR`, `12 kOhms` (RES-1).
- `SIExpr::parse` grammar for unit expressions (EXPR-1, not evaluated yet).
