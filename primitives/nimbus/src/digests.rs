//! Aura (Authority-Round) digests
//!
//! This implements the digests for AuRa, to allow the private
//! `CompatibleDigestItem` trait to appear in public interfaces.

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