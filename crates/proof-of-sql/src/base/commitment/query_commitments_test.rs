//! Tests for QueryCommitments.

#[cfg(test)]
mod query_commitments_test {
    use crate::base::commitment::query_commitments::QueryCommitments;
    use crate::base::commitment::Commitment;

    #[test]
    fn test_query_commitments_type_exists() {
        let _: Option<QueryCommitments<crate::base::commitment::NaiveCommitment>> = None;
    }

    #[test]
    fn test_query_commitments_is_index_map() {
        // QueryCommitments is an alias for IndexMap
        use crate::base::IndexMap;
        let map: QueryCommitments<crate::base::commitment::NaiveCommitment> = IndexMap::default();
        assert_eq!(map.len(), 0);
    }
}
