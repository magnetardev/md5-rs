pub fn dummy() -> u8 {
    0
}

#[cfg(test)]
mod test {
    #[test]
    fn dummy_test() {
        assert_eq!(0, super::dummy());
    }
}
