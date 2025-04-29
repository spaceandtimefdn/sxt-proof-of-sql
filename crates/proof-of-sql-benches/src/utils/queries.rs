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

/// Filter query.
pub struct Filter;
impl BaseEntry for Filter {
    fn title(&self) -> &'static str {
        "Filter"
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
pub struct ComplexFilter;
impl BaseEntry for ComplexFilter {
    fn title(&self) -> &'static str {
        "Complex Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM bench_table WHERE (((a = $1) AND (b = $2)) OR ((c = $3) AND (d = $4)))"
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
            ("d", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![
            LiteralValue::BigInt(0),
            LiteralValue::BigInt(1),
            LiteralValue::VarChar("a".to_string()),
            LiteralValue::VarChar("b".to_string()),
        ]
    }
}

/// Group by query.
pub struct GroupBy;
impl BaseEntry for GroupBy {
    fn title(&self) -> &'static str {
        "Group By"
    }

    fn sql(&self) -> &'static str {
        "SELECT SUM(a), COUNT(*) FROM bench_table WHERE a = $1 GROUP BY b"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::Int,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("b", ColumnType::VarChar, None),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::Int(0)]
    }
}

/// Join query.
pub struct Join;
impl BaseEntry for Join {
    fn title(&self) -> &'static str {
        "Join"
    }

    fn sql(&self) -> &'static str {
        "SELECT bench_table.a, bench_table_2.a 
         FROM bench_table 
         JOIN bench_table_2 on bench_table.a=bench_table_2.a"
    }

    fn columns(&self) -> Vec<ColumnDefinition> {
        vec![
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10 * size).max(10) as i64),
            ),
        ]
    }

    fn params(&self) -> Vec<LiteralValue> {
        vec![LiteralValue::BigInt(0)]
    }
}

/// Retrieves all available queries.
pub fn all_queries() -> Vec<QueryEntry> {
    vec![
        Filter.entry(),
        ComplexFilter.entry(),
        GroupBy.entry(),
        Join.entry(),
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
