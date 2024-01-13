#[cfg(test)]

mod test_email_validation {

    use crate::convert_tools::ConvertTools;

    static TEST_INIT: std::sync::Once = std::sync::Once::new();

    fn init() {
        TEST_INIT.call_once(|| {
        });
    }

    #[test]
    fn test_mail_validation() {
        init();

        let test_input="a <>#%+{}|\\^~[]';/?:@=&$".to_string();
        let test_awaited="a%20%3C%3E%23%25%2B%7B%7D%7C%5C%5E%7E%5B%5D%60%3B%2F%3F%3A%40%3D%26%24";

        let conversion_value = ConvertTools::escape_htmltext(&test_input);

        assert_eq!(test_awaited,conversion_value);
        assert_ne!(test_input,conversion_value);

    }
}