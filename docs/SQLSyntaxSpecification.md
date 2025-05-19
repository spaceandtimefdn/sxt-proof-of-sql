# PoSQL SQL Syntax

Proof of SQL uses [sqlparser](https://github.com/apache/datafusion-sqlparser-rs) to parse SQL. It currently supports the following syntax. The syntax support is rapidly expanding, and we are happy to take suggestions about what should be added. Anyone submitting a PR must ensure that this is kept up to date.

| **Category**             | **Feature**               | **Prover** | **EVM Verification** | **Post-Processing** |
|--------------------------|---------------------------|:----------:|:--------------------:|:-------------------:|
| **DataTypes**            | Bool / Boolean            | ✅        | ✅                  | ✅                 |
|                          | Uint8 (8 bits)            | ✅        | ✅                  | ✅                 |
|                          | TinyInt (8 bits)          | ✅        | ✅                  | ✅                 |
|                          | SmallInt (16 bits)        | ✅        | ✅                  | ✅                 |
|                          | Int / Integer (32 bits)   | ✅        | ✅                  | ✅                 |
|                          | BigInt (64 bits)          | ✅        | ✅                  | ✅                 |
|                          | Int128                    | ✅        | ✅                  | ✅                 |
|                          | Decimal75                 | ✅        | ✅                  | ✅                 |
|                          | Varchar[^1]               | ✅        | ❌                  | ✅                 |
|                          | Varbinary[^1]             | ✅        | ❌                  | ✅                 |
|                          | Timestamp                 | ✅        | ✅                  | ✅                 |
| **Operators**            | AND, OR                   | ✅        | ✅                  | ✅                 |
|                          | NOT                       | ✅        | ✅                  | ✅                 |
|                          | +, –, *                   | ✅        | ✅                  | ✅                 |
|                          | /                         | ❌        | ❌                  | ✅                 |
|                          | =, !=                     | ✅        | ✅                  | ✅                 |
|                          | >, ≥, <, ≤                | ✅        | ❌                  | ✅                 |
| **Aggregate Functions**  | SUM                       | ✅        | ❌                  | ✅                 |
|                          | COUNT                     | ✅        | ❌                  | ✅                 |
| **SELECT Syntax**        | WHERE clause              | ✅        | ✅                  | ✅                 |
|                          | GROUP BY clause           | ✅        | ❌                  | ✅                 |
|                          | LIMIT clause              | ✅        | ❌                  | ✅                 |
|                          | OFFSET clause             | ✅        | ❌                  | ✅                 |
|                          | UNION ALL operator        | ✅        | ❌                  | ✅                 |
|                          | JOIN clause[^2]           | ✅        | ❌                  | ✅                 |


[^1]: Currently, we do not support any string or binary operations beyond = and !=.
[^2]: Currently, we only support some inner joins on one column.

For more details please refer to [DataFusion SELECT syntax](https://datafusion.apache.org/user-guide/sql/select.html).

## Reserved keywords

The following keywords may not be used as aliases:
- `count`
