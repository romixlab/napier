# si_dynamic features and roadmap

This file is the single source of truth for what si_dynamic does, what is broken and what is planned, for humans
and AI agents alike. [CHANGELOG.md](CHANGELOG.md) records what changed and when; this file records the current
state.

Last full review: 3 Oct 2026 (commit `b14b5aa`, 0.1.0). Built from the code and the tests; the bugs marked
*(verified 3 Oct 2026)* were reproduced with a throwaway test on that date.

## How to use this file

- **Status** of each item:
  - ✅ done
  - 🚧 in progress or partially done (the note says what is missing)
  - 🐛 implemented, but with known bugs
  - ⬜ stub: types or API exist but do nothing yet
  - 📋 planned
  - 💡 idea, not committed to
  - ⛔ blocked (the note says on what)
  - 🔍 probably done or obsolete, needs a check before closing
- **IDs** (`UNIT-2`, `NUM-3`) are stable: never renumber or reuse one. New items take the next free number of their
  area. Use the ID in commit messages, CHANGELOG entries and code `TODO`s (`// TODO(EXPR-1): ...`).
- Items are grouped by area. Each area lists what works first, then open items by priority.
- When finishing work, update the item in the same commit: mark it ✅, add a pointer (function, grammar rule,
  test) and move it up to the done items of its area. Don't delete done items. Items that turn out obsolete go to
  [Dropped and superseded](#dropped-and-superseded) with a one-line reason.
- A bug you find but don't fix gets an entry (🐛 on the feature, or a new item) with the input that triggers it.
- Small code-level gaps stay as `TODO` comments; only those that limit users or block a feature get an item.

## Units and prefixes (`UNIT`)

- ✅ **UNIT-1 SI base and derived units**: the 7 base units (gram in place of kilogram, UNIT-6) and 22 derived ones
  (`BaseUnit`, `KNOWN_UNITS`), each with `name()`, `symbol()`, `dim()` (`Dim`) and `scale()` (`Scale`).
- ✅ **UNIT-2 All SI prefixes**: quecto to quetta, `u` / `μ` / `µ` for micro, `k` / `K` for kilo, split off a word
  in `parse::resolve_word` (UNIT-12).
- ✅ **UNIT-3 Unit exponents**: `m^2`, superscripts `m²`, `s⁻¹` (exponent fits in `i8`). Superscripts failed to
  parse before 0.3 (`parse_superscript` in `crates/si_dynamic/src/parse.rs`).
- ✅ **UNIT-4 Unknown units**: any identifier the parser and the registry don't know parses as
  `BaseUnit::Unknown` (`10mVDC` → milli + `VDC`), with no dimension (`dim()` is `None`).
- ✅ **UNIT-5 Serde**: `Unit`, `UnitExpr`, `BaseUnit`, `Prefix`, `Dim`, `Scale`, `Quantity`, `NamedUnit` derive
  `Serialize` / `Deserialize`. The shape of `Prefix`, `Unit` and `UnitExpr` is pinned by golden JSON
  (`crates/si_dynamic/tests/golden/units.json`, `crates/si_dynamic/tests/golden.rs`) and changes only with a new
  major version (README, *Serde
  stability*).
- ✅ **UNIT-6 Gram as the base of mass**: `BaseUnit::Gram` with `Scale::Pow10(-3)`, `10 kg` is kilo + gram
  (test `gram_is_the_base_of_mass`). Was: `g` mapped to `Kilogram`, `10 kg` became `kkg`.
- ✅ **UNIT-7 Degree Celsius affine**: `°C` has `Scale::Affine { factor: 1, offset: 273.15 }`; `to_si_f64` and
  `UnitExpr::convert_f64` take the offset for a single °C, a compound unit with °C (`ppm/°C`) has none and doesn't
  convert to SI (test `celsius_is_affine`). `℃` (U+2103) is read as `°C`, `m°C` is milli + °C (UNIT-13).
- 📋 **UNIT-8 Superscript exponents in `Display`**: `Unit` prints `m^2` instead of `m²` (TODO in `Display for
  Unit`). Check rvariant's round-trip tests when changing it.
- ✅ **UNIT-9 Named units registered at run time**: `Registry::register(NamedUnit { name, symbol, dim, scale,
  aliases })` and `Registry::alias(word, Unit)` (`Vdc` = volt, `ppb` = nano + ratio); `Registry::parse_quantity` /
  `parse_unit`. SI symbols can't be redefined. A `BaseUnit::Named` value carries its dimension and scale, so stored
  values convert without the registry (`crates/si_dynamic/src/registry.rs`, test `registry`).
- ✅ **UNIT-10 Unit model after ww_si**: `Dim` (s, m, kg, A, K, mol, cd, ww_si's order) in place of `SIExp`,
  `Scale::{Coherent, Pow10, Factor, Affine}`, `UnitExpr::{Unit, Mul, Div, Pow}` with `dim()`, `factor()`,
  `offset()`, `exp10()`, `prefix_exp10()`, `without_prefixes()`, `convert_f64()` (`crates/si_dynamic/src/dim.rs`,
  `crates/si_dynamic/src/unit.rs`).
- ✅ **UNIT-11 Ratios**: `BaseUnit::Ratio` for dimensionless numbers; `%`, `‰`, `ppm` parse as centi, milli and
  micro + ratio and are written back that way (test `ratios`). A number without unit is a ratio.
- ✅ **UNIT-12 Whole known units win over a prefix split**: `Pa`, `cd`, `mol`, `kat`, `Gy`, `ppm` and registered
  words (`rpm`, `ppb`) are the unit, not a prefix + rest; then a prefix + known unit (`kPa`, `mcd`, `dam`, `krpm`);
  unknown words keep the old split (`mVDC`). A ratio or prefixed alias takes no further prefix (`k%`, `kppm` fail)
  (`parse::resolve_word`, test `whole_known_units_win_over_a_prefix_split`). Was: per-unit grammar rules with
  look-ahead, `ppb` split as pico + `pb`.
- ✅ **UNIT-13 Prefixes before non-ASCII units**: `100 mΩ`, `10 uΩ`, `4.7 kΩ` with Ω as U+03A9 or U+2126, `µ`
  (U+00B5) and `μ` (U+03BC) for micro before any unit, `m°C` (test `prefixes_before_non_ascii_units`). Was:
  `100 mΩ` failed (the look-ahead wanted an ASCII letter), `µΩ` worked only through a precedence slip in the micro
  rule, `10 µ` failed while `10 u` parsed.

## Numbers and quantities (`NUM`)

- ✅ **NUM-1 `Quantity::parse`**: number plus optional unit, spaces optional (`10V`, `4.7 uF`, ` 10 V `). Keeps the
  number as text, `as_f32()` applies the prefix.
- ✅ **NUM-2 Unicode minus** (`−`, U+2212) and `-` for negative numbers.
- ✅ **NUM-3 Prefix raised to the exponent**: `1 km^2` is 1e6 m² (`Unit::prefix_exp10`, test
  `prefix_raised_to_the_exponent`).
- ✅ **NUM-4 Exponent notation without a unit**: `1e3`, `2.5e-3` (test `exponent_without_unit`). Was: failed
  with `Internal("expected unit")`.
- ✅ **NUM-5 `f64` and exact values**: `Quantity::value_f64`, `as_f64`, `as_f32` (unit without prefixes),
  `to_si_f64` (coherent SI), exact `base_decimal` (decimal point moved as text) and `base_i64` (test
  `f64_and_exact_values`).
- 📋 **NUM-6 Error messages**: `Error::Internal` strings name grammar rules (`"expected simple_number"`), not what
  was wrong with the input. Give users a message they can act on, with the position.

## Resistance (`RES`)

- ✅ **RES-1 `OhmF32::parse`**: plain numbers, prefixes as decimal point (`4k7`, `1k2`), `R` / `r` / `Ω` / `Ohm` /
  `Ohms`, prefixes `k M G T m u n` with or without the unit (`5k`, `499kR`, `12 kOhms`, `1 mΩ`).
- ✅ **RES-2 Exponents in resistance**: `OhmF32::parse("1e3")`, `"1.5e3"`, `"2e-3"`, `"2.2e3 Ω"`; the number is
  read as decimal text and shifted exactly (`parse_any_number_f32`, test `resistance_exponents`). Was: failed with
  `Internal("simple_number")`, negative exponents rejected.

## Unit expressions (`EXPR`)

- ✅ **EXPR-1 Unit expressions**: `Quantity::parse` and `Registry::parse_unit` read `10 V/us`, `10*V/us`,
  `1 m/s²`, `ppm/°C`, `kg·m/s^2`, `W/(m⋅K)`, `(m/s)^2` into `UnitExpr` (`Mul`, `Div`, `Pow`), left associative;
  `*`, `⋅` (U+22C5) and `·` (U+00B7) multiply; `Display` parses back to the same expression (test
  `unit_expressions`). Replaces the `SIExpr` stub.
- ✅ **EXPR-2 Dimension check**: `UnitExpr::dim()` reduces an expression to a `Dim` (`V/A` is Ω), `convert_f64`
  converts between units of one dimension (`V/mA` to `kΩ`).

## Project (`PRJ`)

- ✅ **PRJ-1 Published on crates.io** (0.1.0).
- 📋 **PRJ-2 README and docs**: crate-level docs with examples of accepted inputs; doc comments on the public
  types.
- 📋 **PRJ-3 Tests**: 5 unit tests (`crates/si_dynamic/src/lib.rs`), `expr()` is all commented out. Add tests for
  every bug above
  before fixing it, and a table test of accepted / rejected inputs.
- ✅ **PRJ-4 Cargo workspace**: the repo is now a workspace (`Cargo.toml`, `[workspace.package]`,
  `[workspace.dependencies]`); si_dynamic moved to `crates/si_dynamic` unchanged, `crates/napier` added as an
  empty library crate for the expression language over si_dynamic's values.

## Dropped and superseded

(none yet)
