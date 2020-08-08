mod dynamic {
    #[cfg(feature = "unit-duration")]
    mod duration {
        use crate::unit::{Duration, Unit};

        #[test]
        fn value_and_upper_bound_use_own_unit() {
            assert_eq!(
                format!("{}", Unit::dynamic(Duration).display(40, Some(300))),
                "40s of 5m"
            );
        }
    }
    #[cfg(feature = "unit-human")]
    mod human {
        use crate::unit::{human, Human, Mode, Unit};

        #[test]
        fn various_combinations() {
            let unit = Unit::dynamic_and_mode(
                Human::new(
                    {
                        let mut f = human::Formatter::new();
                        f.with_decimals(1);
                        f
                    },
                    "objects",
                ),
                Mode::PercentageAfterUnit,
            );
            assert_eq!(
                format!("{}", unit.display(100_002, Some(7_500_000))),
                "100.0k/7.5M objects [1%]"
            );
            assert_eq!(format!("{}", unit.display(100_002, None)), "100.0k objects");
        }
    }
    mod range {
        use crate::unit::{Mode, Range, Unit};
        #[test]
        fn value_and_upper_bound_with_percentage() {
            let unit = Unit::dynamic_and_mode(Range::new("steps"), Mode::PercentageAfterUnit);
            assert_eq!(format!("{}", unit.display(0, Some(3))), "1 of 3 steps [0%]");
            assert_eq!(format!("{}", unit.display(1, Some(3))), "2 of 3 steps [33%]");
            assert_eq!(format!("{}", unit.display(2, Some(3))), "3 of 3 steps [66%]");
        }
    }
    #[cfg(feature = "unit-bytes")]
    mod bytes {
        use crate::unit::{Bytes, Mode, Unit};

        #[test]
        fn value_and_upper_bound_use_own_unit() {
            assert_eq!(
                format!(
                    "{}",
                    Unit::dynamic_and_mode(Bytes, Mode::PercentageAfterUnit).display(1002, Some(10_000_000_000))
                ),
                "1.0KB/10.0GB [0%]"
            );
        }
        #[test]
        fn just_value() {
            assert_eq!(format!("{}", Unit::dynamic(Bytes).display(5540, None)), "5.5KB");
        }
    }
}

mod label {
    mod with_percentage {
        mod only_values {
            use crate::unit::{Mode, Unit};
            #[test]
            fn display_current_value_with_upper_bound_percentage_before_value() {
                assert_eq!(
                    format!(
                        "{}",
                        Unit::label_and_mode("items", Mode::PercentageBeforeValue)
                            .display(123, Some(400))
                            .values()
                    ),
                    "[30%] 123/400"
                );
            }
        }

        mod only_unit {
            use crate::unit::{Mode, Unit};
            #[test]
            fn display_current_value_with_upper_bound_percentage_after_unit() {
                assert_eq!(
                    format!(
                        "{}",
                        Unit::label_and_mode("items", Mode::PercentageAfterUnit)
                            .display(123, Some(400))
                            .unit()
                    ),
                    "items [30%]"
                );
            }
        }
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
