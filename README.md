# napier

A Cargo workspace with two crates.

- [crates/si_dynamic](crates/si_dynamic) parses SI quantities and units from text at runtime: `4.7 uF`, `10mV`,
  `4k7` (resistance), units with prefixes and exponents and unit expressions like `m/s²`. Published on crates.io
  as `si_dynamic`.
- [crates/napier](crates/napier) is the expression language over si_dynamic's values, named after John Napier.
  Empty for now.

What works, known bugs and plans: [FEATURES.md](FEATURES.md). Changes: [CHANGELOG.md](CHANGELOG.md).

License: MIT.
