use validator::ValidateUrl;

#[derive(Debug)]
pub struct LongUrl(String);

impl LongUrl {
    pub fn parse(s: String) -> Result<LongUrl, String> {
        if s.trim().is_empty() {
            return Err("URL cannot be empty.".to_string());
        }
        if !ValidateUrl::validate_url(&s) {
            return Err(format!("'{s}' is not a valid URL."));
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for LongUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::LongUrl;
    use claims::{assert_err, assert_ok};

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(LongUrl::parse(email));
    }

    #[test]
    fn correct_url_format_is_accepted() {
        let email = "http://localhost.com".to_string();
        assert_ok!(LongUrl::parse(email));
    }

    #[test]
    fn non_url_is_rejected() {
        let email = "hello".to_string();
        assert_err!(LongUrl::parse(email));
    }
}
