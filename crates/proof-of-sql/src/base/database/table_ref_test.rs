#[cfg(test)]
mod tests {
    use crate::base::database::{ParseError, TableRef};
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_create_table_ref_with_schema_and_name() {
        let t = TableRef::new("sxt", "users");
        assert_eq!(t.schema_id().map(|id| id.value.as_str()), Some("sxt"));
        assert_eq!(t.table_id().value.as_str(), "users");
    }

    #[test]
    fn we_can_create_table_ref_with_empty_schema() {
        let t = TableRef::new("", "products");
        assert!(t.schema_id().is_none());
        assert_eq!(t.table_id().value.as_str(), "products");
    }

    #[test]
    fn we_can_create_table_ref_from_names() {
        let t = TableRef::from_names(Some("sxt"), "orders");
        assert_eq!(t.to_string(), "sxt.orders");

        let t2 = TableRef::from_names(None, "users");
        assert_eq!(t2.to_string(), "users");
    }

    #[test]
    fn we_can_create_table_ref_from_idents() {
        let schema = Ident::new("sxt");
        let table = Ident::new("logs");
        let t = TableRef::from_idents(Some(schema), table);
        assert_eq!(t.to_string(), "sxt.logs");

        let t2 = TableRef::from_idents(None, Ident::new("metrics"));
        assert_eq!(t2.to_string(), "metrics");
    }

    #[test]
    fn we_can_create_table_ref_from_strs() {
        let t = TableRef::from_strs(&["users"]).unwrap();
        assert_eq!(t.to_string(), "users");

        let t2 = TableRef::from_strs(&["sxt", "orders"]).unwrap();
        assert_eq!(t2.to_string(), "sxt.orders");

        let t3 = TableRef::from_strs::<&str>(&["a", "b", "c"]);
        assert!(t3.is_err());
    }

    #[test]
    fn we_can_create_table_ref_try_from_str() {
        let t: TableRef = "users".try_into().unwrap();
        assert_eq!(t.to_string(), "users");

        let t2: TableRef = "sxt.orders".try_into().unwrap();
        assert_eq!(t2.to_string(), "sxt.orders");

        let t3: Result<TableRef, _> = "a.b.c".try_into();
        assert!(matches!(t3, Err(ParseError::InvalidTableReference { .. })));
    }

    #[test]
    fn we_can_create_table_ref_from_str() {
        let t: TableRef = "users".parse().unwrap();
        assert_eq!(t.to_string(), "users");

        let t2: TableRef = "sxt.logs".parse().unwrap();
        assert_eq!(t2.to_string(), "sxt.logs");

        let t3: Result<TableRef, _> = "x.y.z".parse();
        assert!(t3.is_err());
    }

    #[test]
    fn we_can_display_table_ref() {
        assert_eq!(TableRef::new("sxt", "users").to_string(), "sxt.users");
        assert_eq!(TableRef::new("", "products").to_string(), "products");
    }

    #[test]
    fn we_can_serialize_and_deserialize_table_ref() {
        let t = TableRef::new("sxt", "users");
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(json, "\"sxt.users\"");

        let deserialized: TableRef = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, t);

        let t2 = TableRef::new("", "metrics");
        let json2 = serde_json::to_string(&t2).unwrap();
        assert_eq!(json2, "\"metrics\"");
        let deserialized2: TableRef = serde_json::from_str(&json2).unwrap();
        assert_eq!(deserialized2, t2);
    }

    #[test]
    fn table_ref_equivalent_works_correctly() {
        use indexmap::Equivalent;
        let t1 = TableRef::new("sxt", "users");
        let t2 = TableRef::new("sxt", "users");
        let t3 = TableRef::new("sxt", "orders");
        let t4 = TableRef::new("", "users");

        assert!(Equivalent::equivalent(&&t1, &t2));
        assert!(Equivalent::equivalent(&&t2, &t1));
        assert!(!Equivalent::equivalent(&&t1, &t3));
        assert!(!Equivalent::equivalent(&&t1, &t4));
    }
}
