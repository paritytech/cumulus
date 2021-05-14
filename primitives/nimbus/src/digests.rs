//! A convenient interface over the digests used in nimbus.
//! 
//! Currently Nimbus has two digests;
//! 1. A consensus digest that contains the block author identity
//!    This information is copied from the author inehrent.
//!    This may be replaced with a pre-runtime digest in the future.
//! 2. A seal digest that contains a signature over the rest of the
//!    block including the first digest.

use crate::{NIMBUS_ENGINE_ID, NimbusSignature, NimbusId};
use sp_runtime::generic::DigestItem;
use parity_scale_codec::{Encode, Codec};
use sp_std::fmt::Debug;

/// A digest item which is usable with aura consensus.
pub trait CompatibleDigestItem: Sized {
	/// Construct a seal digest item from the given signature
	fn nimbus_seal(signature: NimbusSignature) -> Self;

	/// If this item is a nimbus seal, return the signature.
	fn as_nimbus_seal(&self) -> Option<NimbusSignature>;

	/// Construct a consensus digest from the given AuthorId
	fn nimbus_consensus_digest(author: NimbusId) -> Self;

	/// If this item is a nimbus  consensus digest, return the author
	fn as_nimbus_consensus_digest(&self) -> Option<NimbusId>;
}

impl<Hash> CompatibleDigestItem for DigestItem<Hash> where
	Hash: Debug + Send + Sync + Eq + Clone + Codec + 'static
{
	fn nimbus_seal(signature: NimbusSignature) -> Self {
		DigestItem::Seal(NIMBUS_ENGINE_ID, signature.encode())
	}

	fn as_nimbus_seal(&self) -> Option<NimbusSignature> {
		self.seal_try_to(&NIMBUS_ENGINE_ID)
	}

	fn nimbus_consensus_digest(author: NimbusId) -> Self {
		DigestItem::Consensus(NIMBUS_ENGINE_ID, author.encode())
	}

	fn as_nimbus_consensus_digest(&self) -> Option<NimbusId> {
		self.pre_runtime_try_to(&NIMBUS_ENGINE_ID)
	}
}