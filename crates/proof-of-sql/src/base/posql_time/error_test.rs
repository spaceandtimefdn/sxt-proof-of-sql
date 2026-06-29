use super::PoSQLTimestampError;

#[test]
fn posql_timestamp_error_display_messages() {
    let err = PoSQLTimestampError::InvalidTimezone {
        timezone: "UTC".to_string(),
    };
    assert_eq!(format!("{}"), "invalid timezone string: UTC");
    
    let err2 = PoSQLTimestampError::InvalidTimezone {
        timezone: "America/New_York".to_string(),
    };
    assert_eq!(format!("{}"), "invalid timezone string: America/New_York");
    
    let err3 = PoSQLTimestampError::InvalidTimezoneOffset;
    assert_eq!(format!("{}"), "invalid timezone offset");
    
    let err4 = PoSQLTimestampError::InvalidTimeUnit {
        error: "invalid unit".to_string(),
    };
    assert_eq!(format!("{}"), "Invalid time unit");
    
    let err5 = PoSQLTimestampError::UnsupportedPrecision {
        error: "invalid precision".to_string(),
    };
    assert_eq!(format!("{}"), "Unsupported precision for timestamp: invalid precision");
}

#[test]
fn posql_timestamp_error_debug_formatting() {
    let err = PoSQLTimestampError::InvalidTimezone {
        timezone: "PST".to_string(),
    };
    assert_eq!(format!("{:?}"), "InvalidTimezone { timezone: "PST" }");
    
    let err2 = PoSQLTimestampError::InvalidTimezoneOffset;
    assert_eq!(format!("{:?}"), "InvalidTimezoneOffset");
}

#[test]
fn posql_timestamp_error_equality() {
    let err1 = PoSQLTimestampError::InvalidTimezone {
        timezone: "UTC".to_string(),
    };
    let err2 = PoSQLTimestampError::InvalidTimezone {
        timezone: "UTC".to_string(),
    };
    let err3 = PoSQLTimestampError::InvalidTimezone {
        timezone: "EST".to_string(),
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}

#[test]
fn posql_timestamp_error_to_string_conversion() {
    let err = PoSQLTimestampError::InvalidTimezone {
        timezone: "test".to_string(),
    };
    let s: String = err.into();
    assert_eq!(s, "invalid timezone string: test");
}
