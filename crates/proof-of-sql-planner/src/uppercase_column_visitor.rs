use sqlparser::ast::{visit_relations_mut, Expr, Ident, Statement, VisitMut, VisitorMut};
use std::ops::ControlFlow;

/// Returns an uppercased version of Ident
/// Leaving this as public because the sdk also uses this function
#[must_use]
#[expect(clippy::needless_pass_by_value)]
pub fn uppercase_identifier(ident: Ident) -> Ident {
    let value = ident.value.to_uppercase();
    Ident { value, ..ident }
}

struct UppercaseColumnVisitor;

impl VisitorMut for UppercaseColumnVisitor {
    type Break = ();

    fn post_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<Self::Break> {
        match expr {
            Expr::Identifier(ident) => *ident = uppercase_identifier(ident.clone()),
            Expr::CompoundIdentifier(idents) => {
                for ident in idents.iter_mut() {
                    *ident = uppercase_identifier(ident.clone());
                }
            }
            _ => (),
        }
        ControlFlow::Continue(())
    }
}

/// Returns the sqlparser statement with all of its column/table identifiers uppercased.
pub fn statement_with_uppercase_identifiers(mut statement: Statement) -> Statement {
    let _ = statement.visit(&mut UppercaseColumnVisitor);

    // uppercase all tables
    let _ = visit_relations_mut(&mut statement, |object_name| {
        object_name.0.iter_mut().for_each(|ident| {
            ident.value = ident.value.to_uppercase();
        });

        ControlFlow::<()>::Continue(())
    });

    statement
}

#[cfg(test)]
mod tests {
    use super::{statement_with_uppercase_identifiers, uppercase_identifier};
    use sqlparser::{ast::Ident, dialect::GenericDialect, parser::Parser};

    #[test]
    fn we_can_capitalize_statement_idents() {
        let statement = Parser::parse_sql(&GenericDialect{}, "SELECT a.thissum from (SELECT Sum(uppercase_Value) as thissum, COUNT(puppies) as coUNT fRoM NonSEnSE) as a").unwrap()[0].clone();
        let statement = statement_with_uppercase_identifiers(statement);
        let expected_statement = Parser::parse_sql(&GenericDialect{}, "SELECT A.THISSUM from (SELECT Sum(UPPERCASE_VALUE) as thissum, COUNT(PUPPIES) as coUNT fRoM NONSENSE) as a").unwrap()[0].clone();
        assert_eq!(statement, expected_statement);
    }

    #[test]
    fn uppercase_identifier_converts_lowercase_to_upper() {
        let ident = Ident::new("hello");
        let result = uppercase_identifier(ident);
        assert_eq!(result.value, "HELLO");
    }

    #[test]
    fn uppercase_identifier_leaves_already_uppercase_unchanged() {
        let ident = Ident::new("WORLD");
        let result = uppercase_identifier(ident);
        assert_eq!(result.value, "WORLD");
    }

    #[test]
    fn uppercase_identifier_handles_mixed_case() {
        let ident = Ident::new("myColumn");
        let result = uppercase_identifier(ident);
        assert_eq!(result.value, "MYCOLUMN");
    }

    #[test]
    fn statement_with_uppercase_identifiers_uppercases_table_names() {
        let statement =
            Parser::parse_sql(&GenericDialect {}, "SELECT id FROM employees")
                .unwrap()[0]
                .clone();
        let result = statement_with_uppercase_identifiers(statement);
        let expected =
            Parser::parse_sql(&GenericDialect {}, "SELECT ID FROM EMPLOYEES")
                .unwrap()[0]
                .clone();
        assert_eq!(result, expected);
    }

    #[test]
    fn statement_with_uppercase_identifiers_handles_schema_qualified_table() {
        let statement =
            Parser::parse_sql(&GenericDialect {}, "SELECT col FROM schema_name.table_name")
                .unwrap()[0]
                .clone();
        let result = statement_with_uppercase_identifiers(statement);
        let expected =
            Parser::parse_sql(&GenericDialect {}, "SELECT COL FROM SCHEMA_NAME.TABLE_NAME")
                .unwrap()[0]
                .clone();
        assert_eq!(result, expected);
    }

    #[test]
    fn statement_with_uppercase_identifiers_handles_where_clause() {
        let statement =
            Parser::parse_sql(&GenericDialect {}, "SELECT id FROM users WHERE active = true")
                .unwrap()[0]
                .clone();
        let result = statement_with_uppercase_identifiers(statement);
        let expected =
            Parser::parse_sql(&GenericDialect {}, "SELECT ID FROM USERS WHERE ACTIVE = true")
                .unwrap()[0]
                .clone();
        assert_eq!(result, expected);
    }
}
