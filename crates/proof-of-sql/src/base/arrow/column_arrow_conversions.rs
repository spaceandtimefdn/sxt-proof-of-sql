use crate::base::{
    database::{ColumnField, ColumnType},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
};
use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, TimeUnit as ArrowTimeUnit};

/// Convert [`ColumnType`] values to some arrow [`DataType`]
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::Boolean => DataType::Boolean,
            ColumnType::Uint8 => DataType::UInt8,
            ColumnType::TinyInt => DataType::Int8,
            ColumnType::SmallInt => DataType::Int16,
            ColumnType::Int => DataType::Int32,
            ColumnType::BigInt => DataType::Int64,
            ColumnType::Int128 => DataType::Decimal128(38, 0),
            ColumnType::Decimal75(precision, scale) => {
                DataType::Decimal256(precision.value(), *scale)
            }
            ColumnType::VarChar => DataType::Utf8,
            ColumnType::VarBinary => DataType::LargeBinary,
            ColumnType::Scalar => unimplemented!("Cannot convert Scalar type to arrow type"),
            ColumnType::TimestampTZ(timeunit, timezone) => {
                let arrow_timezone = Some(Arc::from(timezone.to_string()));
                let arrow_timeunit = match timeunit {
                    PoSQLTimeUnit::Second => ArrowTimeUnit::Second,
                    PoSQLTimeUnit::Millisecond => ArrowTimeUnit::Millisecond,
                    PoSQLTimeUnit::Microsecond => ArrowTimeUnit::Microsecond,
                    PoSQLTimeUnit::Nanosecond => ArrowTimeUnit::Nanosecond,
                };
                DataType::Timestamp(arrow_timeunit, arrow_timezone)
            }
        }
    }
}

/// Convert arrow [`DataType`] values to some [`ColumnType`]
impl TryFrom<DataType> for ColumnType {
    type Error = String;

    fn try_from(data_type: DataType) -> Result<Self, Self::Error> {
        match data_type {
            DataType::Boolean => Ok(ColumnType::Boolean),
            DataType::UInt8 => Ok(ColumnType::Uint8),
            DataType::Int8 => Ok(ColumnType::TinyInt),
            DataType::Int16 => Ok(ColumnType::SmallInt),
            DataType::Int32 => Ok(ColumnType::Int),
            DataType::Int64 => Ok(ColumnType::BigInt),
            DataType::Decimal128(38, 0) => Ok(ColumnType::Int128),
            DataType::Decimal256(precision, scale) if precision <= 75 => {
                Ok(ColumnType::Decimal75(Precision::new(precision)?, scale))
            }
            DataType::Timestamp(time_unit, timezone_option) => {
                let posql_time_unit = match time_unit {
                    ArrowTimeUnit::Second => PoSQLTimeUnit::Second,
                    ArrowTimeUnit::Millisecond => PoSQLTimeUnit::Millisecond,
                    ArrowTimeUnit::Microsecond => PoSQLTimeUnit::Microsecond,
                    ArrowTimeUnit::Nanosecond => PoSQLTimeUnit::Nanosecond,
                };
                Ok(ColumnType::TimestampTZ(
                    posql_time_unit,
                    PoSQLTimeZone::try_from(&timezone_option)?,
                ))
            }
            DataType::Utf8 => Ok(ColumnType::VarChar),
            DataType::LargeBinary => Ok(ColumnType::VarBinary),
            _ => Err(format!("Unsupported arrow data type {data_type:?}")),
        }
    }
}
/// Convert [`ColumnField`] values to arrow Field
impl From<&ColumnField> for Field {
    fn from(column_field: &ColumnField) -> Self {
        Field::new(
            column_field.name().value.as_str(),
            (&column_field.data_type()).into(),
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    #[should_panic(expected = "Cannot convert Scalar type to arrow type")]
    fn we_cannot_convert_scalar_column_type_to_arrow() {
        let _ = DataType::from(&ColumnType::Scalar);
    }

    #[test]
    fn we_can_convert_timestamp_column_types_to_arrow_and_back() {
        let cases = [
            (PoSQLTimeUnit::Second, ArrowTimeUnit::Second),
            (PoSQLTimeUnit::Millisecond, ArrowTimeUnit::Millisecond),
            (PoSQLTimeUnit::Microsecond, ArrowTimeUnit::Microsecond),
            (PoSQLTimeUnit::Nanosecond, ArrowTimeUnit::Nanosecond),
        ];
        let timezone = PoSQLTimeZone::new(-19_800);

        for (posql_timeunit, arrow_timeunit) in cases {
            let column_type = ColumnType::TimestampTZ(posql_timeunit, timezone);
            let arrow_type = DataType::from(&column_type);

            assert_eq!(
                arrow_type,
                DataType::Timestamp(arrow_timeunit, Some(Arc::from("-05:30")))
            );
            assert_eq!(ColumnType::try_from(arrow_type).unwrap(), column_type);
        }
    }

    #[test]
    fn we_cannot_convert_unsupported_arrow_data_type_to_column_type() {
        assert_eq!(
            ColumnType::try_from(DataType::Float32).unwrap_err(),
            "Unsupported arrow data type Float32"
        );
    }

    proptest! {
        #[test]
        fn we_can_roundtrip_arbitrary_column_type(column_type: ColumnType) {
            let arrow = DataType::from(&column_type);
            let actual = ColumnType::try_from(arrow).unwrap();

            prop_assert_eq!(actual, column_type);
        }
    }
}
