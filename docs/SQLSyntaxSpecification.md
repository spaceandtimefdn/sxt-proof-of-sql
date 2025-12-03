# PoSQL SQL Syntax

Proof of SQL uses [sqlparser](https://github.com/apache/datafusion-sqlparser-rs) to parse SQL. It currently supports the following syntax. The syntax support is rapidly expanding, and we are happy to take suggestions about what should be added. Anyone submitting a PR must ensure that this is kept up to date.

| **Category**             | **Feature**               | **Prover** | **EVM Verification** |
|--------------------------|---------------------------|:----------:|:--------------------:|
| **DataTypes**            | Bool / Boolean            | ✅        | ✅                  |
|                          | Uint8 (8 bits)            | ✅        | ✅                  |
|                          | TinyInt (8 bits)          | ✅        | ✅                  |
|                          | SmallInt (16 bits)        | ✅        | ✅                  |
|                          | Int / Integer (32 bits)   | ✅        | ✅                  |
|                          | BigInt (64 bits)          | ✅        | ✅                  |
|                          | Int128                    | ✅        | ✅                  |
|                          | Decimal75[^4]                 | ✅        | ✅                  |
|                          | Varchar[^1]               | ✅        | ✅                  |
|                          | Varbinary[^1]             | ✅        | ✅                  |
|                          | Timestamp                 | ✅        | ✅                  |
| **Operators**            | AND, OR                   | ✅        | ✅                  |
|                          | NOT                       | ✅        | ✅                  |
|                          | +, –, *                   | ✅        | ✅                  |
|                          | /                         | ❌        | ❌                  |
|                          | =, !=                     | ✅        | ✅                  |
|                          | >, ≥, <, ≤                | ✅        | ✅                  |
| **Aggregate Functions**[^3]  | SUM                       | ✅        | ✅                  |
|                          | COUNT                     | ✅        | ✅                  |
| **SELECT Syntax**        | WHERE clause              | ✅        | ✅                  |
|                          | GROUP BY clause           | ✅        | ✅                  |
|                          | LIMIT clause              | ✅        | ✅                  |
|                          | OFFSET clause             | ✅        | ✅                  |
|                          | UNION ALL operator        | ✅        | ✅                  |
|                          | JOIN clause[^2]           | ✅        | ✅                  |


[^1]: Currently, we do not support any string or binary operations beyond = and !=.
[^2]: Currently, we only support some inner joins on one column.
[^3]: Currently there are restrictions on aggregations we support.
[^4]: Currently, we only support decimals up to 75 digits of precision and inequality operators only operate on decimals up to 38 digits of precision.

For more details please refer to [DataFusion SELECT syntax](https://datafusion.apache.org/user-guide/sql/select.html).

## Reserved keywords

The following keywords may not be used as aliases:
- `count`
