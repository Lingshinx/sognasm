#[cfg(test)]
mod tests {
    use crate::util::align_8;

    #[test]
    fn test_align_8() {
        assert_eq!(0, align_8(0));
        assert_eq!(8, align_8(7));
        assert_eq!(8, align_8(8));
        assert_eq!(232, align_8(231));
    }
}
