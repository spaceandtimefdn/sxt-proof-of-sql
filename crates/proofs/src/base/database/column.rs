use super::TableRef;
use crate::base::scalar::ArkScalar;
use arrow::datatypes::{DataType, Field};
use proofs_sql::Identifier;
use serde::{Deserialize, Serialize};

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Column<'a> {
    /// i64 columns
    BigInt(&'a [i64]),
    /// String columns
    ///  - the first element maps to the str values.
    ///  - the second element maps to the str hashes (see [crate::base::scalar::ArkScalar]).
    VarChar((&'a [&'a str], &'a [ArkScalar])),
    /// i128 columns
    Int128(&'a [i128]),
    /// Scalar columns
    #[cfg(test)]
    Scalar(&'a [ArkScalar]),
}

impl Column<'_> {
    /// Provides the column type associated with the column
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
            #[cfg(test)]
            Self::Scalar(_) => ColumnType::Scalar,
        }
    }
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            Self::BigInt(col) => col.len(),
            Self::VarChar((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::Int128(col) => col.len(),
            #[cfg(test)]
            Self::Scalar(col) => col.len(),
        }
    }
    /// Returns `true` if the column has no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The precision for [ColumnType::INT128] values
pub const INT128_PRECISION: usize = 38;

/// The scale for [ColumnType::INT128] values
pub const INT128_SCALE: usize = 0;

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize, Copy)]
pub enum ColumnType {
    /// Mapped to i64
    #[serde(alias = "BIGINT", alias = "bigint")]
    BigInt,
    /// Mapped to i128
    #[serde(rename = "Decimal", alias = "DECIMAL", alias = "decimal")]
    Int128,
    /// Mapped to String
    #[serde(alias = "VARCHAR", alias = "varchar")]
    VarChar,
    /// Mapped to ArkScalar
    #[serde(alias = "SCALAR", alias = "scalar")]
    #[cfg(test)]
    Scalar,
}

/// Convert ColumnType values to some arrow DataType
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::BigInt => DataType::Int64,
            ColumnType::Int128 => DataType::Decimal128(38, 0),
            ColumnType::VarChar => DataType::Utf8,
            #[cfg(test)]
            ColumnType::Scalar => unimplemented!("Cannot convert Scalar type to arrow type"),
        }
    }
}

/// Convert arrow DataType values to some ColumnType
impl TryFrom<DataType> for ColumnType {
    type Error = String;

    fn try_from(data_type: DataType) -> Result<Self, Self::Error> {
        match data_type {
            DataType::Int64 => Ok(ColumnType::BigInt),
            DataType::Decimal128(38, 0) => Ok(ColumnType::Int128),
            DataType::Utf8 => Ok(ColumnType::VarChar),
            _ => Err(format!("Unsupported arrow data type {:?}", data_type)),
        }
    }
}

/// Display the column type as a str name (in all caps)
impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnType::BigInt => write!(f, "BIGINT"),
            ColumnType::Int128 => write!(f, "DECIMAL"),
            ColumnType::VarChar => write!(f, "VARCHAR"),
            #[cfg(test)]
            ColumnType::Scalar => write!(f, "SCALAR"),
        }
    }
}

/// Parse the column type from a str name (flexible about case)
impl std::str::FromStr for ColumnType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BIGINT" => Ok(ColumnType::BigInt),
            "DECIMAL" => Ok(ColumnType::Int128),
            "VARCHAR" => Ok(ColumnType::VarChar),
            #[cfg(test)]
            "SCALAR" => Ok(ColumnType::Scalar),
            _ => Err(format!("Unsupported column type {:?}", s)),
        }
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, Serialize, Deserialize)]
pub struct ColumnRef {
    column_id: Identifier,
    table_ref: TableRef,
    column_type: ColumnType,
}

impl ColumnRef {
    /// Create a new `ColumnRef` from a table, column identifier and column type
    pub fn new(table_ref: TableRef, column_id: Identifier, column_type: ColumnType) -> Self {
        Self {
            column_id,
            column_type,
            table_ref,
        }
    }

    /// Returns the table reference of this column
    pub fn table_ref(&self) -> TableRef {
        self.table_ref
    }

    /// Returns the column identifier of this column
    pub fn column_id(&self) -> Identifier {
        self.column_id
    }

    /// Returns the column type of this column
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }
}

/// This type is used to represent the metadata
/// of a column in a table. Namely: it's name and type.
///
/// This is the analog of a `Field` in Apache Arrow.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy, Serialize, Deserialize)]
pub struct ColumnField {
    name: Identifier,
    data_type: ColumnType,
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    pub fn new(name: Identifier, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
    }

    /// Returns the name of the column
    pub fn name(&self) -> Identifier {
        self.name
    }

    /// Returns the type of the column
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }
}

/// Convert ColumnField values to arrow Field
impl From<&ColumnField> for Field {
    fn from(column_field: &ColumnField) -> Self {
        Field::new(
            column_field.name().name(),
            (&column_field.data_type()).into(),
            false,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_type_serializes_to_string() {
        let column_type = ColumnType::BigInt;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""BigInt""#);

        let column_type = ColumnType::Int128;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Decimal""#);

        let column_type = ColumnType::VarChar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""VarChar""#);

        let column_type = ColumnType::Scalar;
        let serialized = serde_json::to_string(&column_type).unwrap();
        assert_eq!(serialized, r#""Scalar""#);
    }

    #[test]
    fn we_can_deserialize_columns_from_valid_strings() {
        let expected_column_type = ColumnType::BigInt;
        let deserialized: ColumnType = serde_json::from_str(r#""BigInt""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Int128;
        let deserialized: ColumnType = serde_json::from_str(r#""DECIMAL""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::VarChar;
        let deserialized: ColumnType = serde_json::from_str(r#""VarChar""#).unwrap();
        assert_eq!(deserialized, expected_column_type);

        let expected_column_type = ColumnType::Scalar;
        let deserialized: ColumnType = serde_json::from_str(r#""SCALAR""#).unwrap();
        assert_eq!(deserialized, expected_column_type);
    }

    #[test]
    fn we_can_deserialize_columns_from_lowercase_or_uppercase_strings() {
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""bigint""#).unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""BIGINT""#).unwrap(),
            ColumnType::BigInt
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""decimal""#).unwrap(),
            ColumnType::Int128
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""DECIMAL""#).unwrap(),
            ColumnType::Int128
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""VARCHAR""#).unwrap(),
            ColumnType::VarChar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""varchar""#).unwrap(),
            ColumnType::VarChar
        );

        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""SCALAR""#).unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            serde_json::from_str::<ColumnType>(r#""scalar""#).unwrap(),
            ColumnType::Scalar
        );
    }

    #[test]
    fn we_cannot_deserialize_columns_from_invalid_strings() {
        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Bigint""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""DecImal""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""Varchar""#);
        assert!(deserialized.is_err());

        let deserialized: Result<ColumnType, _> = serde_json::from_str(r#""ScaLar""#);
        assert!(deserialized.is_err());
    }

    #[test]
    fn we_can_convert_columntype_to_string_and_back_with_display_and_parse() {
        assert_eq!(format!("{}", ColumnType::BigInt), "BIGINT");
        assert_eq!(format!("{}", ColumnType::Int128), "DECIMAL");
        assert_eq!(format!("{}", ColumnType::VarChar), "VARCHAR");
        assert_eq!(format!("{}", ColumnType::Scalar), "SCALAR");
        assert_eq!("BIGINT".parse::<ColumnType>().unwrap(), ColumnType::BigInt);
        assert_eq!("DECIMAL".parse::<ColumnType>().unwrap(), ColumnType::Int128);
        assert_eq!(
            "VARCHAR".parse::<ColumnType>().unwrap(),
            ColumnType::VarChar
        );
        assert_eq!("SCALAR".parse::<ColumnType>().unwrap(), ColumnType::Scalar);
    }

    #[test]
    fn we_can_get_the_len_of_a_column() {
        let scals = [ArkScalar::from(1), ArkScalar::from(2), ArkScalar::from(3)];

        let column = Column::BigInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::VarChar((&["a", "b", "c"], &scals));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::Int128(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::Scalar(&scals);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::BigInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::VarChar((&[], &[]));
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::Int128(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::Scalar(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
    }
}
