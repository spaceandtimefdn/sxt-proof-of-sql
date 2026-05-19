use super::{
    NullableQueryData, NullableQueryResult, ProofPlan, QueryData, QueryProof, QueryResult,
};
use crate::{
    base::{
        commitment::CommitmentEvaluationProof,
        database::{
            ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, LiteralValue,
            NullableOwnedTable, OwnedTable,
        },
        proof::PlaceholderResult,
        scalar::Scalar,
    },
    sql::proof::QueryError,
    utils::log,
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// The result of an sql query along with a proof that the query is valid. The
/// result and proof can be verified using commitments to database columns.
///
/// Note: the query result is stored in an intermediate form rather than the final form
/// the end-user sees. The final form is obtained after verification. Using an
/// intermediate form allows us to handle overflow and certain cases where the final
/// result might use floating point numbers (e.g. `SELECT STDDEV(A) FROM T WHERE B = 0`).
///
/// Below we demonstrate typical usage of [`VerifiableQueryResult`] with pseudo-code.
///
/// Here we assume that a verifier only has access to the commitments of database columns. To
/// process a query, the verifier forwards the query to an untrusted
/// prover. The prover has full access to the database and constructs a [`VerifiableQueryResult`] that
/// it sends back to the verifier. The verifier checks that the result is valid using its
/// commitments, and constructs the finalized form of the query result.
///
/// ```ignore
/// prover_process_query(database_accessor) {
///       query <- receive_query_from_verifier()
///
///       verifiable_result <- VerifiableQueryResult::new(query, database_accessor)
///             // When we construct VerifiableQueryResult from a query expression, we compute
///             // both the result of the query in intermediate form and the proof of the result
///             // at the same time.
///
///       send_to_verifier(verifiable_result)
/// }
///
/// verifier_process_query(query, commitment_accessor) {
///    verifiable_result <- send_query_to_prover(query)
///
///    verify_result <- verifiable_result.verify(query, commitment_accessor)
///    if verify_result.is_error() {
///         // The prover did something wrong. Perhaps the prover tried to tamper with the query
///         // result or maybe its version of the database was out-of-sync with the verifier's
///         // version.
///         do_verification_error()
///    }
///
///    query_result <- verify_result.query_result()
///    if query_result.is_error() {
///         // The prover processed the query correctly, but the query resulted in an error.
///         // For example, perhaps the query added two 64-bit integer columns together that
///         // resulted in an overflow.
///         do_query_error()
///    }
///
///    do_query_success(query_result)
///         // The prover correctly processed a query and the query succeeded. Now, we can
///         // proceed to use the result.
/// }
/// ```
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Clone, Serialize, Deserialize)]
pub struct VerifiableQueryResult<CP: CommitmentEvaluationProof> {
    /// The result of the query in intermediate form.
    pub result: OwnedTable<CP::Scalar>,
    /// The proof that the query result is valid.
    pub proof: QueryProof<CP>,
}

impl<CP: CommitmentEvaluationProof> VerifiableQueryResult<CP> {
    /// Form a `VerifiableQueryResult` from a query expression.
    ///
    /// This function both computes the result of a query and constructs a proof of the results
    /// validity.
    #[tracing::instrument(name = "VerifiableQueryResult::new", level = "info", skip_all)]
    pub fn new(
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup<'_>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Self> {
        log::log_memory_usage("Start");
        let (proof, res) = QueryProof::new(expr, accessor, setup, params)?;
        log::log_memory_usage("End");
        Ok(Self { result: res, proof })
    }

    /// Verify a `VerifiableQueryResult`. Upon success, this function returns the finalized form of
    /// the query result.
    ///
    /// Note: a verified result can still respresent an error (e.g. overflow), but it is a verified
    /// error.
    ///
    /// Nullable results are verified against their physical value-plus-presence columns, then
    /// returned as the compatibility value-only [`OwnedTable`]. Use [`Self::verify_nullable`] to
    /// preserve nullable presence data.
    #[tracing::instrument(name = "VerifiableQueryResult::verify", level = "info", skip_all)]
    pub fn verify(
        self,
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        setup: &CP::VerifierPublicSetup<'_>,
        params: &[LiteralValue],
    ) -> QueryResult<CP::Scalar> {
        log::log_memory_usage("Start");
        let QueryData {
            table,
            verification_hash,
        } = self
            .proof
            .verify(expr, accessor, self.result, setup, params)?;
        Ok(QueryData {
            table: coerce_physical_table_to_logical_values(
                table,
                expr.get_column_result_fields(),
                expr.get_column_result_fields_with_presence(),
            )?,
            verification_hash,
        })
    }

    /// Verify a `VerifiableQueryResult` and return nullable columns reassembled from the
    /// physical value-plus-presence result.
    #[tracing::instrument(
        name = "VerifiableQueryResult::verify_nullable",
        level = "info",
        skip_all
    )]
    pub fn verify_nullable(
        self,
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        setup: &CP::VerifierPublicSetup<'_>,
        params: &[LiteralValue],
    ) -> NullableQueryResult<CP::Scalar> {
        log::log_memory_usage("Start");
        let QueryData {
            table,
            verification_hash,
        } = self
            .proof
            .verify(expr, accessor, self.result, setup, params)?;
        let physical_table =
            table.try_coerce_with_fields(expr.get_column_result_fields_with_presence())?;
        let result = NullableQueryData {
            table: NullableOwnedTable::try_from_values_and_presence_table_with_fields(
                physical_table,
                expr.get_column_result_fields(),
            )?,
            verification_hash,
        };
        log::log_memory_usage("End");
        Ok(result)
    }
}

fn coerce_physical_table_to_logical_values<S: Scalar>(
    table: OwnedTable<S>,
    logical_fields: Vec<ColumnField>,
    physical_fields: Vec<ColumnField>,
) -> Result<OwnedTable<S>, QueryError> {
    let mut physical_columns = table
        .try_coerce_with_fields(physical_fields)?
        .into_inner()
        .into_iter();
    let logical_columns = logical_fields
        .into_iter()
        .map(|field| {
            let (name, column) = physical_columns
                .next()
                .expect("Coerced physical table should include every logical value column");
            debug_assert_eq!(name, field.name());
            if field.is_nullable() {
                let (presence_name, _) = physical_columns
                    .next()
                    .expect("Coerced physical table should include nullable presence columns");
                debug_assert_eq!(presence_name, ColumnRef::presence_column_id(&name));
            }
            Ok((name, column))
        })
        .collect::<Result<Vec<_>, QueryError>>()?;
    debug_assert!(physical_columns.next().is_none());

    Ok(OwnedTable::try_from_iter(logical_columns)?)
}
