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
    fn we_can_convert_column_fields_to_arrow_fields() {
        let column_field = ColumnField::new(
            "amount".into(),
            ColumnType::Decimal75(Precision::new(75).unwrap(), -3),
        );

        assert_eq!(
            Field::from(&column_field),
            Field::new("amount", DataType::Decimal256(75, -3), false)
        );
    }

    #[test]
    fn we_reject_unsupported_arrow_data_types() {
        assert_eq!(
            ColumnType::try_from(DataType::Float32),
            Err("Unsupported arrow data type Float32".to_string())
        );
        assert_eq!(
            ColumnType::try_from(DataType::Decimal128(37, 0)),
            Err("Unsupported arrow data type Decimal128(37, 0)".to_string())
        );
        assert_eq!(
            ColumnType::try_from(DataType::Decimal256(76, 0)),
            Err("Unsupported arrow data type Decimal256(76, 0)".to_string())
        );
    }

    #[test]
    #[should_panic(expected = "not implemented: Cannot convert Scalar type to arrow type")]
    fn we_panic_when_converting_scalar_column_type_to_arrow_type() {
        let _ = DataType::from(&ColumnType::Scalar);
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
