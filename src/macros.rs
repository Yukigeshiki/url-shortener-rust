#[macro_export]
macro_rules! impl_json_display {
    ($strct:ty) => {
        impl Display for $strct {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_fmt(format_args!(
                    "{}",
                    serde_json::to_string(self).unwrap_or("Not available".into())
                ))
            }
        }
    };
}
