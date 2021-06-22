#[macro_export]
macro_rules! test_reader {
    ($reader:ident) => {
        #[test]
        fn basic_test_1() {
            crate::testing::basic_test_1::<$reader>();
        }

        #[test]
        fn next_n_bytes_test() {
            crate::testing::next_n_bytes_test::<$reader>();
        }

        #[test]
        fn basic_le_ref_test() {
            crate::testing::basic_le_test::<$reader>();
        }

        #[test]
        fn basic_be_ref_test() {
            crate::testing::basic_be_test::<$reader>();
        }

        #[test]
        fn test_sliceable_retain_offset() {
            crate::testing::test_sliced_retain_offset::<$reader>();
        }

        #[test]
        fn test_advance_by() {
            crate::testing::test_advance_by::<$reader>();
        }

        #[test]
        fn test_advance_to() {
            crate::testing::test_advance_to::<$reader>();
        }
    };
}
