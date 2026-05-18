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
    fn we_can_uppercase_identifier_while_preserving_quote_style() {
        let ident = Ident {
            value: "mixedCase".into(),
            quote_style: Some('"'),
        };

        let result = uppercase_identifier(ident);

        assert_eq!(result.value, "MIXEDCASE");
        assert_eq!(result.quote_style, Some('"'));
    }

    #[test]
    fn we_can_capitalize_statement_idents() {
        let statement = Parser::parse_sql(&GenericDialect{}, "SELECT a.thissum from (SELECT Sum(uppercase_Value) as thissum, COUNT(puppies) as coUNT fRoM NonSEnSE) as a").unwrap()[0].clone();
        let statement = statement_with_uppercase_identifiers(statement);
        let expected_statement = Parser::parse_sql(&GenericDialect{}, "SELECT A.THISSUM from (SELECT Sum(UPPERCASE_VALUE) as thissum, COUNT(PUPPIES) as coUNT fRoM NONSENSE) as a").unwrap()[0].clone();
        assert_eq!(statement, expected_statement);
    }

    #[test]
    fn we_can_capitalize_schema_qualified_statement_idents() {
        let statement = Parser::parse_sql(
            &GenericDialect {},
            "SELECT db.tbl.id, other_schema.other_table.value FROM db.tbl JOIN other_schema.other_table ON db.tbl.id = other_schema.other_table.id",
        )
        .unwrap()[0]
            .clone();

        let statement = statement_with_uppercase_identifiers(statement);
        let expected_statement = Parser::parse_sql(
            &GenericDialect {},
            "SELECT DB.TBL.ID, OTHER_SCHEMA.OTHER_TABLE.VALUE FROM DB.TBL JOIN OTHER_SCHEMA.OTHER_TABLE ON DB.TBL.ID = OTHER_SCHEMA.OTHER_TABLE.ID",
        )
        .unwrap()[0]
            .clone();

        assert_eq!(statement, expected_statement);
    }
}
