//! From the pest parse tree to [Quantity] and [UnitExpr].

use crate::unit::{BaseUnit, Prefix, Unit, UnitExpr};
use crate::{Error, Quantity, Registry};
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/si.pest"]
pub(crate) struct SIParser;

pub(crate) fn expected_rule(name: &'static str) -> Error {
    Error::Internal(name.to_string())
}

pub(crate) fn quantity(s: &str, units: &Registry) -> Result<Quantity, Error> {
    let mut p = SIParser::parse(Rule::si_number, s)?
        .next()
        .ok_or(expected_rule("si_number"))?
        .into_inner();
    let number = p.next().ok_or(expected_rule("number"))?.as_str();
    let unit = match p.find(|p| p.as_rule() == Rule::unit_expr) {
        Some(e) => parse_unit_expr(e, units)?,
        None => UnitExpr::one(),
    };
    Ok(Quantity {
        number: number.to_string(),
        unit,
    })
}

pub(crate) fn unit_expr(s: &str, units: &Registry) -> Result<UnitExpr, Error> {
    let e = SIParser::parse(Rule::unit_only, s)?
        .next()
        .ok_or(expected_rule("unit_only"))?
        .into_inner()
        .next()
        .ok_or(expected_rule("unit_expr"))?;
    parse_unit_expr(e, units)
}

/// `a / b · c` from left to right: `(a / b) · c`.
fn parse_unit_expr(p: Pair<Rule>, units: &Registry) -> Result<UnitExpr, Error> {
    if p.as_rule() != Rule::unit_expr {
        return Err(expected_rule("unit_expr"));
    }
    let mut p = p.into_inner();
    let mut e = parse_operand(p.next().ok_or(expected_rule("unit"))?, units)?;
    while let Some(verb) = p.next() {
        let rhs = parse_operand(p.next().ok_or(expected_rule("unit"))?, units)?;
        let is_div = verb.into_inner().next().map(|v| v.as_rule()) == Some(Rule::div);
        e = if is_div { e / rhs } else { e * rhs };
    }
    Ok(e)
}

fn parse_operand(p: Pair<Rule>, units: &Registry) -> Result<UnitExpr, Error> {
    match p.as_rule() {
        Rule::unit => Ok(UnitExpr::Unit(parse_unit(p, units)?)),
        Rule::group => {
            let mut p = p.into_inner();
            let e = parse_unit_expr(p.next().ok_or(expected_rule("unit_expr"))?, units)?;
            Ok(match p.next() {
                Some(n) => e.pow(parse_exponent(n)?),
                None => e,
            })
        }
        _ => Err(expected_rule("unit or group")),
    }
}

fn parse_exponent(p: Pair<Rule>) -> Result<i8, Error> {
    let exp = match p.as_rule() {
        Rule::ss_number => parse_superscript(p.as_str())?,
        _ => parse_integer_number_i64(p)?,
    };
    i8::try_from(exp).map_err(|_| Error::Internal("unit exponent must fit in i8".into()))
}

fn parse_unit(p: Pair<Rule>, units: &Registry) -> Result<Unit, Error> {
    if p.as_rule() != Rule::unit {
        return Err(expected_rule("unit"));
    }
    let mut p = p.into_inner();
    let word = p.next().ok_or(expected_rule("word"))?.as_str();
    let mut unit = resolve_word(word, units)?;
    if let Some(num) = p.next() {
        unit.exp = parse_exponent(num)?
            .checked_mul(unit.exp)
            .ok_or(Error::Internal("unit exponent must fit in i8".into()))?;
    }
    Ok(unit)
}

/// Prefix symbols, `da` before `d`.
const PREFIXES: [(&str, Prefix); 27] = [
    ("da", Prefix::Deca),
    ("Q", Prefix::Quetta),
    ("R", Prefix::Ronna),
    ("Y", Prefix::Yotta),
    ("Z", Prefix::Zetta),
    ("E", Prefix::Exa),
    ("P", Prefix::Peta),
    ("T", Prefix::Tera),
    ("G", Prefix::Giga),
    ("M", Prefix::Mega),
    ("k", Prefix::Kilo),
    ("K", Prefix::Kilo),
    ("h", Prefix::Hecto),
    ("d", Prefix::Deci),
    ("c", Prefix::Centi),
    ("m", Prefix::Milli),
    ("u", Prefix::Micro),
    ("\u{00B5}", Prefix::Micro),
    ("\u{03BC}", Prefix::Micro),
    ("n", Prefix::Nano),
    ("p", Prefix::Pico),
    ("f", Prefix::Femto),
    ("a", Prefix::Atto),
    ("z", Prefix::Zepto),
    ("y", Prefix::Yocto),
    ("r", Prefix::Ronto),
    ("q", Prefix::Quecto),
];

/// The unit a word stands for: a whole known unit (`Pa`, `cd`, `ppm`, a registered word) wins over a prefix split
/// (UNIT-12); else a prefix and a known unit (`kPa`, `mΩ`, `µF`, `krpm`, UNIT-13); else, as before 0.3, a prefix and
/// an unknown ASCII word (`mVDC` is milli + `VDC`), or the whole word unknown.
fn resolve_word(word: &str, units: &Registry) -> Result<Unit, Error> {
    let word = word.replace('\u{2126}', "Ω").replace('\u{2103}', "°C");
    if let Some(unit) = units.lookup(&word) {
        return Ok(unit);
    }
    let split = PREFIXES
        .iter()
        .find_map(|(sym, prefix)| Some((*prefix, word.strip_prefix(sym)?)))
        .filter(|(_, rest)| !rest.is_empty());
    let Some((prefix, rest)) = split else {
        return Ok(Unit::base(BaseUnit::Unknown(word)));
    };
    if let Some(unit) = units.lookup(rest) {
        if unit.prefix != Prefix::Unit || unit.base == BaseUnit::Ratio || unit.exp != 1 {
            return Err(Error::Internal(format!(
                "{rest} takes no prefix, {word} is not a unit"
            )));
        }
        return Ok(Unit { prefix, ..unit });
    }
    if rest.starts_with(|c: char| c.is_ascii_alphabetic()) && rest.is_ascii() {
        Ok(Unit::new(prefix, BaseUnit::Unknown(rest.to_string()), 1))
    } else {
        Ok(Unit::base(BaseUnit::Unknown(word)))
    }
}

fn parse_superscript(s: &str) -> Result<i64, Error> {
    let mut v: i64 = 0;
    let mut minus = false;
    for c in s.chars() {
        let d = match c {
            '⁻' => {
                minus = true;
                continue;
            }
            '⁰' => 0,
            '¹' => 1,
            '²' => 2,
            '³' => 3,
            '⁴'..='⁹' => c as i64 - '⁴' as i64 + 4,
            _ => return Err(expected_rule("ss_number")),
        };
        v = v
            .checked_mul(10)
            .and_then(|v| v.checked_add(d))
            .ok_or(Error::Internal("unit exponent must fit in i8".into()))?;
    }
    Ok(if minus { -v } else { v })
}

pub(crate) fn parse_integer_number_i64(rule: Pair<Rule>) -> Result<i64, Error> {
    if rule.as_rule() != Rule::integer_number {
        return Err(expected_rule("integer_number"));
    }
    let mut p = rule.into_inner();
    let minus_or_raw_number = p.next().ok_or(expected_rule("minus or raw_number"))?;
    let is_minus = minus_or_raw_number.as_rule() == Rule::minus;
    let int = if is_minus {
        p.next().ok_or(expected_rule("raw_number"))?.as_str()
    } else {
        minus_or_raw_number.as_str()
    };
    let v: i64 = int.parse()?;
    Ok(if is_minus { -v } else { v })
}

/// A prefix of resistor notation (`4k7`).
pub(crate) fn parse_prefix(rule: Pair<Rule>) -> Result<Prefix, Error> {
    let rule = rule.into_inner().next().ok_or(expected_rule("prefix"))?;
    let prefix = match rule.as_rule() {
        Rule::tera_unconditional => Prefix::Tera,
        Rule::giga_unconditional => Prefix::Giga,
        Rule::mega_unconditional => Prefix::Mega,
        Rule::kilo_unconditional => Prefix::Kilo,
        Rule::milli_unconditional => Prefix::Milli,
        Rule::micro_unconditional => Prefix::Micro,
        Rule::nano_unconditional => Prefix::Nano,
        _ => return Err(expected_rule("prefix")),
    };
    Ok(prefix)
}

/// The decimal number `text` (`-4.7`, `1.5e3`, with `−` for minus) with its point moved by `exp10`, written out
/// without an exponent: exact, no float in between.
pub(crate) fn shift_decimal(text: &str, exp10: i32) -> Option<String> {
    let text = text.trim().replace('\u{2212}', "-");
    let (mantissa, exp) = match text.split_once(['e', 'E']) {
        Some((m, e)) => (m, e.parse::<i32>().ok()?),
        None => (text.as_str(), 0),
    };
    let (negative, mantissa) = match mantissa.strip_prefix('-') {
        Some(m) => (true, m),
        None => (false, mantissa),
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if int.is_empty() && frac.is_empty()
        || !int.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let digits: String = format!("{int}{frac}");
    // value = digits · 10^(point - len)
    let point = int.len() as i64 + exp as i64 + exp10 as i64;
    if point.unsigned_abs() > 1000 {
        return None;
    }
    let len = digits.len() as i64;
    let (int, frac) = if point <= 0 {
        ("0".to_string(), "0".repeat((-point) as usize) + &digits)
    } else if point >= len {
        (digits + &"0".repeat((point - len) as usize), String::new())
    } else {
        let (i, f) = digits.split_at(point as usize);
        (i.to_string(), f.to_string())
    };
    let int = int.trim_start_matches('0');
    let int = if int.is_empty() { "0" } else { int };
    let frac = frac.trim_end_matches('0');
    let zero = int == "0" && frac.is_empty();
    let sign = if negative && !zero { "-" } else { "" };
    Some(if frac.is_empty() {
        format!("{sign}{int}")
    } else {
        format!("{sign}{int}.{frac}")
    })
}
