//! The expression language over si_dynamic's values, named after John Napier.

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
