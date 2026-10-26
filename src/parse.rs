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
    let unit = match p.next() {
        Some(unit) if unit.as_rule() == Rule::unit => UnitExpr::Unit(parse_unit(unit, units)?),
        _ => UnitExpr::one(),
    };
    Ok(Quantity {
        number: number.to_string(),
        unit,
    })
}

pub(crate) fn unit_expr(s: &str, units: &Registry) -> Result<UnitExpr, Error> {
    let unit = SIParser::parse(Rule::unit_only, s)?
        .next()
        .ok_or(expected_rule("unit_only"))?
        .into_inner()
        .next()
        .ok_or(expected_rule("unit"))?;
    Ok(UnitExpr::Unit(parse_unit(unit, units)?))
}

fn parse_unit(p: Pair<Rule>, units: &Registry) -> Result<Unit, Error> {
    if p.as_rule() != Rule::unit {
        return Err(expected_rule("unit"));
    }
    let mut p = p.into_inner();
    let first = p.next().ok_or(expected_rule("prefix or unit"))?;
    let mut unit = match first.as_rule() {
        Rule::ratio_unit => units
            .lookup(first.as_str())
            .ok_or(expected_rule("ratio_unit"))?,
        Rule::prefix => {
            let prefix = parse_prefix(first)?;
            let raw_unit = p.next().ok_or(expected_rule("raw_unit"))?;
            let unit = parse_raw_unit(raw_unit, units)?;
            if unit.prefix != Prefix::Unit {
                return Err(Error::Internal(format!(
                    "{unit} has a prefix of its own, {prefix} can't be added"
                )));
            }
            Unit { prefix, ..unit }
        }
        _ => parse_raw_unit(first, units)?,
    };
    if let Some(num) = p.next() {
        let exp = match num.as_rule() {
            Rule::ss_number => parse_superscript(num.as_str())?,
            _ => parse_integer_number_i64(num)?,
        };
        let exp = i8::try_from(exp)
            .ok()
            .and_then(|e| e.checked_mul(unit.exp))
            .ok_or(Error::Internal("unit exponent must fit in i8".into()))?;
        unit.exp = exp;
    }
    Ok(unit)
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

fn parse_raw_unit(p: Pair<Rule>, units: &Registry) -> Result<Unit, Error> {
    if p.as_rule() != Rule::raw_unit {
        return Err(expected_rule("raw_unit"));
    }
    let raw_unit = p.into_inner().next().ok_or(expected_rule("raw_unit"))?;
    let word = raw_unit.as_str();
    Ok(match raw_unit.as_rule() {
        Rule::ident => units
            .lookup(word)
            .unwrap_or_else(|| Unit::base(BaseUnit::Unknown(word.to_string()))),
        _ => units.lookup(word).ok_or(expected_rule("raw_unit"))?,
    })
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

pub(crate) fn parse_prefix(rule: Pair<Rule>) -> Result<Prefix, Error> {
    let rule = rule.into_inner().next().ok_or(expected_rule("prefix"))?;
    let prefix = match rule.as_rule() {
        Rule::quetta => Prefix::Quetta,
        Rule::ronna => Prefix::Ronna,
        Rule::yotta => Prefix::Yotta,
        Rule::zetta => Prefix::Zetta,
        Rule::exa => Prefix::Exa,
        Rule::peta => Prefix::Peta,
        Rule::tera | Rule::tera_unconditional => Prefix::Tera,
        Rule::giga | Rule::giga_unconditional => Prefix::Giga,
        Rule::mega | Rule::mega_unconditional => Prefix::Mega,
        Rule::kilo | Rule::kilo_unconditional => Prefix::Kilo,
        Rule::hecto => Prefix::Hecto,
        Rule::deca => Prefix::Deca,
        Rule::deci => Prefix::Deci,
        Rule::centi => Prefix::Centi,
        Rule::milli | Rule::milli_unconditional => Prefix::Milli,
        Rule::micro | Rule::micro_unconditional => Prefix::Micro,
        Rule::nano | Rule::nano_unconditional => Prefix::Nano,
        Rule::pico => Prefix::Pico,
        Rule::femto => Prefix::Femto,
        Rule::atto => Prefix::Atto,
        Rule::zepto => Prefix::Zepto,
        Rule::yocto => Prefix::Yocto,
        Rule::ronto => Prefix::Ronto,
        Rule::quecto => Prefix::Quecto,
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
