mod label {
    mod with_percentage {
        use crate::unit::{Mode, Unit};

        #[test]
        fn display_current_value_no_upper_bound_shows_no_percentage() {
            assert_eq!(
                format!(
                    "{}",
                    Unit::label_and_mode("items", Mode::PercentageAfterUnit).display(123, None)
                ),
                "123 items"
            );
        }
        #[test]
        fn display_current_value_with_upper_bound_shows_percentage() {
            assert_eq!(
                format!(
                    "{}",
                    Unit::label_and_mode("items", Mode::PercentageAfterUnit).display(123, Some(500))
                ),
                "123/500 items [24%]"
            );
            assert_eq!(
                format!(
                    "{}",
                    Unit::label_and_mode("items", Mode::PercentageBeforeValue).display(123, Some(500))
                ),
                "[24%] 123/500 items"
            );
        }
    }
    mod without_percentage {
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
}
