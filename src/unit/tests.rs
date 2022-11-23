mod dynamic {
    #[cfg(feature = "unit-duration")]
    mod duration {
        use crate::unit::{self, Duration};

        #[test]
        fn value_and_upper_bound_use_own_unit() {
            assert_eq!(
                format!("{}", unit::dynamic(Duration).display(40, Some(300), None)),
                "40s of 5m"
            );
        }
    }
    #[cfg(feature = "unit-human")]
    mod human {
        use crate::unit::{self, display, human, Human};

        #[test]
        fn various_combinations() {
            let unit = unit::dynamic_and_mode(
                Human::new(
                    {
                        let mut f = human::Formatter::new();
                        f.with_decimals(1);
                        f
                    },
                    "objects",
                ),
                display::Mode::with_percentage(),
            );
            assert_eq!(
                format!("{}", unit.display(100_002, Some(7_500_000), None)),
                "100.0k/7.5M objects [1%]"
            );
            assert_eq!(format!("{}", unit.display(100_002, None, None)), "100.0k objects");
        }
    }
    mod range {
        use crate::unit::{self, display, Range};
        #[test]
        fn value_and_upper_bound_with_percentage() {
            let unit = unit::dynamic_and_mode(Range::new("steps"), display::Mode::with_percentage());
            assert_eq!(format!("{}", unit.display(0, Some(3), None)), "1 of 3 steps [0%]");
            assert_eq!(format!("{}", unit.display(1, Some(3), None)), "2 of 3 steps [33%]");
            assert_eq!(format!("{}", unit.display(2, Some(3), None)), "3 of 3 steps [66%]");
        }
    }
    #[cfg(feature = "unit-bytes")]
    mod bytes {
        use crate::unit::{self, display, Bytes};

        #[test]
        fn value_and_upper_bound_use_own_unit() {
            assert_eq!(
                format!(
                    "{}",
                    unit::dynamic_and_mode(Bytes, display::Mode::with_percentage()).display(
                        1002,
                        Some(10_000_000_000),
                        None
                    )
                ),
                "1.0KB/10.0GB [0%]"
            );
        }
        #[test]
        fn just_value() {
            assert_eq!(format!("{}", unit::dynamic(Bytes).display(5540, None, None)), "5.5KB");
        }
    }
}

mod label {
    mod with_percentage {
        mod only_values {
            use crate::unit::{self, display};
            #[test]
            fn display_current_value_with_upper_bound_percentage_before_value() {
                assert_eq!(
                    format!(
                        "{}",
                        unit::label_and_mode("items", display::Mode::with_percentage().show_before_value())
                            .display(123, Some(400), None)
                            .values()
                    ),
                    "[30%] 123/400"
                );
            }
        }

        mod only_unit {
            use crate::unit::{self, display};
            #[test]
            fn display_current_value_with_upper_bound_percentage_after_unit() {
                assert_eq!(
                    format!(
                        "{}",
                        unit::label_and_mode("items", display::Mode::with_percentage())
                            .display(123, Some(400), None)
                            .unit()
                    ),
                    "items [30%]"
                );
            }
        }
        use std::time;

        use crate::unit::{self, display};

        #[test]
        fn display_current_over_time_shows_throughput() {
            let unit = unit::label_and_mode("items", display::Mode::with_percentage().and_throughput());
            assert_eq!(
                format!("{}", unit.display(123, None, None)),
                "123 items",
                "from one measurement, there can be no throughput"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(
                        500,
                        None,
                        display::Throughput::new(250, time::Duration::from_millis(500))
                    )
                ),
                "500 items |250/500ms|",
                "sub-second intervals are displayed with millisecond precision"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(700, None, display::Throughput::new(500, time::Duration::from_secs(1)))
                ),
                "700 items |500/s|",
                "a '1' in the timespan is not displayed"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(500, None, display::Throughput::new(250, time::Duration::from_secs(30)))
                ),
                "500 items |250/30s|",
                "sub-minute intervals are displayed with second precision"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(700, None, display::Throughput::new(500, time::Duration::from_secs(60)))
                ),
                "700 items |500/m|",
                "it also knows minutes"
            );
            let unit = unit::label_and_mode(
                "items",
                display::Mode::with_percentage().and_throughput().show_before_value(),
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(700, None, display::Throughput::new(500, time::Duration::from_secs(90)))
                ),
                "|500/1.5m| 700 items",
                "it uses fractions on the biggest possible unit"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(
                        500,
                        None,
                        display::Throughput::new(250, time::Duration::from_secs(30 * 60))
                    )
                ),
                "|250/30m| 500 items",
                "sub-hour intervals are displayed with minute precision"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit.display(
                        700,
                        None,
                        display::Throughput::new(500, time::Duration::from_secs(60 * 60))
                    )
                ),
                "|500/h| 700 items",
                "it also knows hours"
            );
        }

        #[test]
        fn display_current_value_no_upper_bound_shows_no_percentage() {
            assert_eq!(
                format!(
                    "{}",
                    unit::label_and_mode("items", display::Mode::with_percentage()).display(123, None, None)
                ),
                "123 items"
            );
        }
        #[test]
        fn display_current_value_with_upper_bound_shows_percentage() {
            assert_eq!(
                format!(
                    "{}",
                    unit::label_and_mode("items", display::Mode::with_percentage()).display(123, Some(500), None)
                ),
                "123/500 items [24%]"
            );
            assert_eq!(
                format!(
                    "{}",
                    unit::label_and_mode("items", display::Mode::with_percentage().show_before_value()).display(
                        123,
                        Some(500),
                        None
                    )
                ),
                "[24%] 123/500 items"
            );
        }
    }
    mod without_percentage {
        use crate::unit;

        #[test]
        fn display_current_value_no_upper_bound() {
            assert_eq!(
                format!("{}", unit::label("items").display(123, None, None)),
                "123 items"
            );
        }
        #[test]
        fn display_current_value_with_upper_bound() {
            assert_eq!(
                format!("{}", unit::label("items").display(123, Some(500), None)),
                "123/500 items"
            );
        }
    }
}

mod size {
    use std::mem::size_of;

    use crate::unit::{display, Unit};

    #[test]
    fn of_mode() {
        assert_eq!(size_of::<display::Mode>(), 3);
    }
    #[test]
    fn of_unit() {
        assert_eq!(size_of::<Unit>(), 32);
    }
}
