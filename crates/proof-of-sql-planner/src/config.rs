use datafusion::config::{ConfigOptions, SqlParserOptions};

/// Since all of our table identifiers/column identifiers are stored and communicated in all-caps,
/// we need to disable this datafusion setting that will coerce identifiers to lowercase.
#[must_use]
pub fn datafusion_config_no_normalization() -> ConfigOptions {
    let mut config = ConfigOptions::new();
    config.sql_parser = SqlParserOptions {
        enable_ident_normalization: false,
        ..Default::default()
    };
    config
}

#[cfg(test)]
mod tests {
    use crate::datafusion_config_no_normalization;

    #[test]
    fn get_config() {
        assert!(
            !datafusion_config_no_normalization()
                .sql_parser
                .enable_ident_normalization
        );
    }
}
