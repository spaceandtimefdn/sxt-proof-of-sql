use super::PoSQLTimestampError;

#[test]
fn we_can_convert_timestamp_errors_to_strings() {
    assert_eq!(
        String::from(PoSQLTimestampError::InvalidTimezone {
            timezone: "Mars/Phobos".to_string()
        }),
        "invalid timezone string: Mars/Phobos"
    );
    assert_eq!(
        String::from(PoSQLTimestampError::InvalidTimezoneOffset),
        "invalid timezone offset"
    );
    assert_eq!(
        String::from(PoSQLTimestampError::InvalidTimeUnit {
            error: "fortnight".to_string()
        }),
        "Invalid time unit"
    );
    assert_eq!(
        String::from(PoSQLTimestampError::UnsupportedPrecision {
            error: "42".to_string()
        }),
        "Unsupported precision for timestamp: 42"
    );
}
