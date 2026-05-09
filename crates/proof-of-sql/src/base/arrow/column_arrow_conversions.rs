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
    fn we_can_convert_supported_column_types_to_arrow_data_types() {
        let precision = Precision::new(12).unwrap();
        let timestamp = ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc());

        assert_eq!(DataType::from(&ColumnType::Boolean), DataType::Boolean);
        assert_eq!(DataType::from(&ColumnType::Uint8), DataType::UInt8);
        assert_eq!(DataType::from(&ColumnType::TinyInt), DataType::Int8);
        assert_eq!(DataType::from(&ColumnType::SmallInt), DataType::Int16);
        assert_eq!(DataType::from(&ColumnType::Int), DataType::Int32);
        assert_eq!(DataType::from(&ColumnType::BigInt), DataType::Int64);
        assert_eq!(
            DataType::from(&ColumnType::Int128),
            DataType::Decimal128(38, 0)
        );
        assert_eq!(
            DataType::from(&ColumnType::Decimal75(precision, -2)),
            DataType::Decimal256(12, -2)
        );
        assert_eq!(DataType::from(&ColumnType::VarChar), DataType::Utf8);
        assert_eq!(
            DataType::from(&ColumnType::VarBinary),
            DataType::LargeBinary
        );
        assert_eq!(
            ColumnType::try_from(DataType::from(&timestamp)).unwrap(),
            timestamp
        );
    }

    #[test]
    fn we_can_convert_supported_arrow_data_types_to_column_types() {
        let precision = Precision::new(75).unwrap();

        assert_eq!(
            ColumnType::try_from(DataType::Boolean).unwrap(),
            ColumnType::Boolean
        );
        assert_eq!(
            ColumnType::try_from(DataType::UInt8).unwrap(),
            ColumnType::Uint8
        );
        assert_eq!(
            ColumnType::try_from(DataType::Int8).unwrap(),
            ColumnType::TinyInt
        );
        assert_eq!(
            ColumnType::try_from(DataType::Int16).unwrap(),
            ColumnType::SmallInt
        );
        assert_eq!(
            ColumnType::try_from(DataType::Int32).unwrap(),
            ColumnType::Int
        );
        assert_eq!(
            ColumnType::try_from(DataType::Int64).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            ColumnType::try_from(DataType::Decimal128(38, 0)).unwrap(),
            ColumnType::Int128
        );
        assert_eq!(
            ColumnType::try_from(DataType::Decimal256(75, 3)).unwrap(),
            ColumnType::Decimal75(precision, 3)
        );
        assert_eq!(
            ColumnType::try_from(DataType::Utf8).unwrap(),
            ColumnType::VarChar
        );
        assert_eq!(
            ColumnType::try_from(DataType::LargeBinary).unwrap(),
            ColumnType::VarBinary
        );
    }

    #[test]
    fn we_can_convert_arrow_timestamp_units_to_column_types() {
        let timezone = Some(Arc::from("UTC"));

        assert_eq!(
            ColumnType::try_from(DataType::Timestamp(ArrowTimeUnit::Second, timezone.clone()))
                .unwrap(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc())
        );
        assert_eq!(
            ColumnType::try_from(DataType::Timestamp(
                ArrowTimeUnit::Millisecond,
                timezone.clone()
            ))
            .unwrap(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc())
        );
        assert_eq!(
            ColumnType::try_from(DataType::Timestamp(
                ArrowTimeUnit::Microsecond,
                timezone.clone()
            ))
            .unwrap(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Microsecond, PoSQLTimeZone::utc())
        );
        assert_eq!(
            ColumnType::try_from(DataType::Timestamp(ArrowTimeUnit::Nanosecond, timezone)).unwrap(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc())
        );
    }

    #[test]
    fn we_get_errors_for_unsupported_arrow_data_types() {
        assert_eq!(
            ColumnType::try_from(DataType::Float32).unwrap_err(),
            "Unsupported arrow data type Float32"
        );
        assert_eq!(
            ColumnType::try_from(DataType::Decimal128(38, 1)).unwrap_err(),
            "Unsupported arrow data type Decimal128(38, 1)"
        );
        assert_eq!(
            ColumnType::try_from(DataType::Decimal256(76, 0)).unwrap_err(),
            "Unsupported arrow data type Decimal256(76, 0)"
        );
    }

    #[test]
    #[should_panic(expected = "not implemented: Cannot convert Scalar type to arrow type")]
    fn we_panic_when_converting_scalar_column_type_to_arrow_data_type() {
        let _ = DataType::from(&ColumnType::Scalar);
    }

    #[test]
    fn we_can_convert_column_field_to_arrow_field() {
        let column_field = ColumnField::new("amount".into(), ColumnType::Int);

        let field = Field::from(&column_field);

        assert_eq!(field.name(), "amount");
        assert_eq!(field.data_type(), &DataType::Int32);
        assert!(!field.is_nullable());
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
