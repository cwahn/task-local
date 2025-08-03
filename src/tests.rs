//! Test that the library works in both std and no_std modes

#[cfg(test)]
mod tests {
    use crate::task_local;

    task_local! {
        static TEST_VALUE: u32;
        static TEST_STRING: &'static str;
    }

    #[test]
    fn test_sync_scope() {
        TEST_VALUE.sync_scope(42, || {
            assert_eq!(TEST_VALUE.get(), 42);
        });

        TEST_STRING.sync_scope("hello", || {
            assert_eq!(TEST_STRING.get(), "hello");
        });
    }

    #[test]
    fn test_nested_scopes() {
        TEST_VALUE.sync_scope(1, || {
            assert_eq!(TEST_VALUE.get(), 1);
            
            TEST_VALUE.sync_scope(2, || {
                assert_eq!(TEST_VALUE.get(), 2);
            });
            
            assert_eq!(TEST_VALUE.get(), 1);
        });
    }

    #[test]
    fn test_try_with_error() {
        let result = TEST_VALUE.try_with(|_| ());
        assert!(result.is_err());
    }

    #[cfg(feature = "std")]
    #[tokio::test]
    async fn test_async_scope() {
        TEST_VALUE.scope(100, async {
            assert_eq!(TEST_VALUE.get(), 100);
        }).await;
    }

    #[cfg(feature = "std")]
    #[tokio::test]
    async fn test_nested_async_scopes() {
        TEST_VALUE.scope(1, async {
            assert_eq!(TEST_VALUE.get(), 1);
            
            TEST_VALUE.scope(2, async {
                assert_eq!(TEST_VALUE.get(), 2);
            }).await;
            
            assert_eq!(TEST_VALUE.get(), 1);
        }).await;
    }
}
