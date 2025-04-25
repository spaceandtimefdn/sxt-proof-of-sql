//! Queries used for benchmarking
//!
//! To add a new query,
//! 1. Implement the `BaseEntry` trait
//! 2. Add the new query to the `all_queries` function.
//! 3. Add the new query to the `proof-of-sql-benches/main.rs` `Query` enum.
//! 4. Add the new query to the `proof-of-sql-benches/main.rs` `Query` `to_string()` impl.
#![expect(clippy::cast_possible_wrap)]
use super::OptionalRandBound;
use proof_of_sql::base::{
    database::{ColumnType, LiteralValue},
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
};

/// Type alias for a single column definition in a query.
type ColumnDefinition = (&'static str, ColumnType, OptionalRandBound);

/// Type alias for a single query entry.
pub type QueryEntry = (
    &'static str,
    &'static str,
    Vec<ColumnDefinition>,
    Vec<LiteralValue>,
);

/// Trait for defining a base query.
pub trait BaseEntry {
    fn title(&self) -> &'static str;
    fn sql(&self) -> &'static str;
    fn columns(&self) -> Vec<ColumnDefinition>;
    fn params(&self) -> Vec<LiteralValue> {
        vec![]
    }
    fn entry(&self) -> QueryEntry {
        (self.title(), self.sql(), self.columns(), self.params())
    }
}

/// Single column filter query.
pub struct SingleColumnFilter;
impl BaseEntry for SingleColumnFilter {
    fn title(&self) -> &'static str {
        "Single Column Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT b FROM bench_table WHERE a = $1"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("b", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::BigInt(0)]
    }
}

/// Multi-column filter query.
pub struct MultiColumnFilter;
impl BaseEntry for MultiColumnFilter {
    fn title(&self) -> &'static str {
        "Multi Column Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM bench_table WHERE ((a = $1) OR (b = $2)) AND (c = $3)"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "b",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("c", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![
            LiteralValue::BigInt(0),
            LiteralValue::BigInt(1),
            LiteralValue::VarChar("a".to_string()),
        ]
    }
}

/// Arithmetic query.
pub struct Arithmetic;
impl BaseEntry for Arithmetic {
    fn title(&self) -> &'static str {
        "Arithmetic"
    }

    fn sql(&self) -> &'static str {
        "SELECT a + b AS r0, a * b - $1 AS r1, c FROM bench_table WHERE a <= b AND a >= $2"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "b",
                ColumnType::TinyInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("c", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::BigInt(2), LiteralValue::BigInt(0)]
    }
}

/// Group by query.
pub struct GroupBy;
impl BaseEntry for GroupBy {
    fn title(&self) -> &'static str {
        "Group By"
    }

    fn sql(&self) -> &'static str {
        "SELECT a, COUNT(*) FROM bench_table WHERE (c = $1) and (a <= b) and (a > $2) GROUP BY a"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "b",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("c", ColumnType::Boolean, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::Boolean(true), LiteralValue::Int(0)]
    }
}

/// Aggregate query.
pub struct Aggregate;
impl BaseEntry for Aggregate {
    fn title(&self) -> &'static str {
        "Aggregate"
    }

    fn sql(&self) -> &'static str {
        "SELECT SUM(a) AS foo, COUNT(1) AS values FROM bench_table WHERE a = b OR c = $1"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "b",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("c", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::VarChar("yz".to_string())]
    }
}

/// Boolean filter query.
pub struct BooleanFilter;
impl BaseEntry for BooleanFilter {
    fn title(&self) -> &'static str {
        "Boolean Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM bench_table WHERE c = $1 and b = $2 or a = $3"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("b", ColumnType::VarChar, None),
            ("c", ColumnType::Boolean, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![
            LiteralValue::Boolean(true),
            LiteralValue::VarChar("xyz".to_string()),
            LiteralValue::BigInt(0),
        ]
    }
}

/// Large column entry query.
pub struct LargeColumnSet;
impl BaseEntry for LargeColumnSet {
    fn title(&self) -> &'static str {
        "Large Column Set"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM bench_table WHERE b = d"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            ("a", ColumnType::Boolean, None),
            (
                "b",
                ColumnType::TinyInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "c",
                ColumnType::SmallInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "d",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "e",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("g", ColumnType::VarChar, None),
            (
                "h",
                ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
                None,
            ),
        ]
    }
}

/// Complex condition query.
pub struct ComplexCondition;
impl BaseEntry for ComplexCondition {
    fn title(&self) -> &'static str {
        "Complex Condition"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM bench_table WHERE (a > c * c AND b < c + $1) OR (d = $2)"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "b",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "c",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("d", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![
            LiteralValue::Int(10),
            LiteralValue::VarChar("xyz".to_string()),
        ]
    }
}

/// Sum Count query.
pub struct SumCount;
impl BaseEntry for SumCount {
    fn title(&self) -> &'static str {
        "Sum Count"
    }

    fn sql(&self) -> &'static str {
        "SELECT SUM(a*b*c) AS foo, SUM(a*b) AS bar, COUNT(1) FROM bench_table WHERE a = $1 OR c-b = $2 AND d = $3"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "b",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            (
                "c",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("d", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![
            LiteralValue::BigInt(0),
            LiteralValue::BigInt(2),
            LiteralValue::VarChar("a".to_string()),
        ]
    }
}

/// Coin query.
pub struct Coin;
impl BaseEntry for Coin {
    fn title(&self) -> &'static str {
        "Coin"
    }

    fn sql(&self) -> &'static str {
        "SELECT 
        SUM( 
        (
            CAST (to_address = $1 as bigint)
            - CAST (from_address = $1 as bigint)
        )
        * value
        * CAST(timestamp AS bigint)
        ) AS weighted_value,
        SUM( 
        (
            CAST (to_address = $1 as bigint)
            - CAST (from_address = $1 as bigint)
        )
        * value
        ) AS total_balance,
        COUNT(1) AS num_transactions
        FROM bench_table;"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            ("from_address", ColumnType::VarChar, None),
            ("to_address", ColumnType::VarChar, None),
            (
                "value",
                ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
                None,
            ),
            (
                "timestamp",
                ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc()),
                None,
            ),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::VarChar("a".to_string())]
    }
}

/// Retrieves all available queries.
pub fn all_queries() -> Vec<QueryEntry> {
    vec![
        SingleColumnFilter.entry(),
        MultiColumnFilter.entry(),
        Arithmetic.entry(),
        GroupBy.entry(),
        Aggregate.entry(),
        BooleanFilter.entry(),
        LargeColumnSet.entry(),
        ComplexCondition.entry(),
        SumCount.entry(),
        Coin.entry(),
    ]
}

/// Retrieves a single query by its title.
///
/// # Arguments
/// * `title` - The title of the query to retrieve.
///
/// # Returns
/// * `Some<QueryEntry>` if the query is found.
/// * `None` if no query with the given title exists.
pub fn get_query(title: &str) -> Option<QueryEntry> {
    all_queries()
        .into_iter()
        .find(|(query_title, _, _, _)| *query_title == title)
}
