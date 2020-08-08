mod label {
    use crate::unit::Unit;

    #[test]
    fn display_current_value_no_upper_bound() {
        assert_eq!(format!("{}", Unit::label("items").display(123, None)), "123 items");
    }
    #[test]
    fn display_current_value_with_upper_bound() {
        assert_eq!(
            format!("{}", Unit::label("items").display(123, Some(500))),
            "123/500 items"
        );
    }
}
