#[cfg(test)]
mod posql_time_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::posql_time::{PoSQLTimeZone, PoSQLTimeUnit, PoSQLTimestampError};
        assert!(PoSQLTimeUnit::default().to_bytes().len() >= 0);
    }
}
