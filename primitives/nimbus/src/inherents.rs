use sp_inherents::{InherentData, InherentIdentifier};
use parity_scale_codec::Encode;

/// The InherentIdentifier for nimbus's author inherent
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"author__";

/// A thing that an outer node could use to inject the inherent data.
/// This should be used in simple uses of the author inherent (eg permissionless authoring)
/// When using the full nimbus system, we are manually inserting the inherent.
pub struct InherentDataProvider<AuthorId>(pub AuthorId);

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl<AuthorId: Encode + Send + Sync> sp_inherents::InherentDataProvider for InherentDataProvider<AuthorId> {
	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		inherent_data.put_data(INHERENT_IDENTIFIER, &self.0)
	}

	async fn try_handle_error(
		&self,
		identifier: &InherentIdentifier,
		_error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		// Dont' process modules from other inherents
		if *identifier != INHERENT_IDENTIFIER {
			return None
		}

		// All errors with the author inehrent are fatal
		Some(Err(sp_inherents::Error::Application(Box::from(String::from("Error processing author inherent")))))
	}
}