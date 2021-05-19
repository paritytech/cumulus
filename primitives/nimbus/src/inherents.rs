use sp_inherents::InherentIdentifier;

//TODO Reconstruct this?
// type InherentType = ...

/// The InherentIdentifier for nimbus's author inherent
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

/// A thing that an outer node could use to inject the inherent data.
/// This should be used in simple uses of the author inherent (eg permissionless authoring)
/// When using the full nimbus system, we are manually inserting the  inherent.
// #[cfg(feature = "std")]
pub struct InherentDataProvider<AuthorId>(pub AuthorId);

// #[cfg(feature = "std")]
// impl<AuthorId: Encode> sp_inherents::InherentDataProvider for InherentDataProvider<AuthorId> {
// 	fn inherent_identifier(&self) -> &'static InherentIdentifier {
// 		&INHERENT_IDENTIFIER
// 	}

// 	fn provide_inherent_data(
// 		&self,
// 		inherent_data: &mut InherentData,
// 	) -> Result<(), sp_inherents::Error> {
// 		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
// 	}

// 	fn error_to_string(&self, error: &[u8]) -> Option<String> {
// 		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
// 	}
// }