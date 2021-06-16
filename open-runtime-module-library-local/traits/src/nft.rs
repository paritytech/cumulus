use codec::FullCodec;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	DispatchResult,
};
use sp_std::fmt::Debug;

/// Abstraction over a non-fungible token system.
#[allow(clippy::upper_case_acronyms)]
pub trait NFT<AccountId> {
	/// The NFT class identifier.
	type ClassId: Default + Copy;

	/// The NFT token identifier.
	type TokenId: Default + Copy;

	/// The balance of account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// The number of NFTs assigned to `who`.
	fn balance(who: &AccountId) -> Self::Balance;

	/// The owner of the given token ID. Returns `None` if the token does not
	/// exist.
	fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<AccountId>;

	/// Transfer the given token ID from one account to another.
	fn transfer(from: &AccountId, to: &AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult;
}
