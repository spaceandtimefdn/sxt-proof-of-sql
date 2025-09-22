use sqlparser::ast::{
    visit_relations_mut, Expr, Ident, ObjectNamePart, Statement, VisitMut, VisitorMut,
};
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
                idents
                    .iter_mut()
                    .for_each(|ident| *ident = uppercase_identifier(ident.clone()));
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
        object_name
            .0
            .iter_mut()
            .for_each(|object_name_part| match object_name_part {
                ObjectNamePart::Identifier(ident) => {
                    ident.value = ident.value.to_uppercase();
                }
            });

        ControlFlow::<()>::Continue(())
    });

    statement
}

#[cfg(test)]
mod tests {
    use super::statement_with_uppercase_identifiers;
    use sqlparser::{dialect::GenericDialect, parser::Parser};

    #[test]
    fn we_can_capitalize_statement_idents() {
        let statement = Parser::parse_sql(&GenericDialect{}, "SELECT a.thissum from (SELECT Sum(uppercase_Value) as thissum, COUNT(puppies) as coUNT fRoM NonSEnSE) as a").unwrap()[0].clone();
        let statement = statement_with_uppercase_identifiers(statement);
        let expected_statement = Parser::parse_sql(&GenericDialect{}, "SELECT A.THISSUM from (SELECT Sum(UPPERCASE_VALUE) as thissum, COUNT(PUPPIES) as coUNT fRoM NONSENSE) as a").unwrap()[0].clone();
        assert_eq!(statement, expected_statement);
    }
}
