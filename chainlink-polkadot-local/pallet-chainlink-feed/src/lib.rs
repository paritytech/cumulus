//! # Chainlink Price Feed Module

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod traits;

pub mod default_weights;
mod utils;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Decode, Encode};
	use frame_support::dispatch::DispatchResultWithPostInfo;
	use frame_support::traits::{Currency, ExistenceRequirement, Get, ReservableCurrency};
	use frame_support::{
		dispatch::{DispatchError, DispatchResult, HasCompact},
		ensure,
		pallet_prelude::*,
		require_transactional,
		weights::Weight,
		PalletId, Parameter, RuntimeDebug,
	};
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::*;
	use sp_arithmetic::traits::BaseArithmetic;
	use sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedSub, Member, One, Saturating, Zero, Bounded};
	use sp_std::convert::{TryFrom, TryInto};
	use sp_std::prelude::*;

	use crate::{
		traits::OnAnswerHandler,
		utils::{median, with_transaction_result},
	};
	use sp_std::ops::Add;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub type RoundId = u32;

	/// The configuration for an oracle feed.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct FeedConfig<
		AccountId: Parameter,
		Balance: Parameter,
		BlockNumber: Parameter,
		Value: Parameter,
	> {
		/// Owner of this feed
		pub owner: AccountId,
		/// The pending owner of this feed
		pub pending_owner: Option<AccountId>,
		/// Value bounds of oracle submissions
		pub submission_value_bounds: (Value, Value),
		/// Count bounds of oracle submissions
		pub submission_count_bounds: (u32, u32),
		/// Payment of oracle rounds
		pub payment: Balance,
		/// Timeout of rounds
		pub timeout: BlockNumber,
		/// Represents the number of decimals with which the feed is configured
		pub decimals: u8,
		/// The description of this feed
		pub description: Vec<u8>,
		/// The round initiation delay
		pub restart_delay: RoundId,
		/// The round oracles are currently reporting data for.
		pub reporting_round: RoundId,
		/// The id of the latest round
		pub latest_round: RoundId,
		/// The id of the first round that contains non-default data
		pub first_valid_round: Option<RoundId>,
		/// The amount of the oracles in this feed
		pub oracle_count: u32,
		/// Number of rounds to keep in storage for this feed.
		pub pruning_window: RoundId,
		/// Keeps track of the round that should be pruned next.
		pub next_round_to_prune: RoundId,
		/// Tracks the amount of debt accumulated by the feed
		/// towards the oracles.
		pub debt: Balance,
		/// The maximum allowed debt a feed can accumulate
		///
		/// If this is a `None` value, the feed is not allowed to accumulate any debt
		pub max_debt: Option<Balance>,
	}

	pub type FeedConfigOf<T> = FeedConfig<
		<T as frame_system::Config>::AccountId,
		BalanceOf<T>,
		<T as frame_system::Config>::BlockNumber,
		<T as Config>::Value,
	>;

	/// Round data relevant to consumers.
	/// Will only be constructed once minimum amount of submissions have
	/// been provided.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct Round<BlockNumber, Value> {
		pub started_at: BlockNumber,
		pub answer: Option<Value>,
		pub updated_at: Option<BlockNumber>,
		pub answered_in_round: Option<RoundId>,
	}

	pub type RoundOf<T> = Round<<T as frame_system::Config>::BlockNumber, <T as Config>::Value>;

	impl<BlockNumber, Value> Round<BlockNumber, Value>
	where
		BlockNumber: Default, // BlockNumber
		Value: Default,       // Value
	{
		/// Create a new Round with the given starting block.
		pub fn new(started_at: BlockNumber) -> Self {
			Self {
				started_at,
				..Default::default()
			}
		}
	}

	/// Round data relevant to oracles.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct RoundDetails<Balance, BlockNumber, Value> {
		pub submissions: Vec<Value>,
		pub submission_count_bounds: (u32, u32),
		pub payment: Balance,
		pub timeout: BlockNumber,
	}

	pub type RoundDetailsOf<T> =
		RoundDetails<BalanceOf<T>, <T as frame_system::Config>::BlockNumber, <T as Config>::Value>;

	/// Meta data tracking withdrawable rewards and admin for an oracle.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct OracleMeta<AccountId, Balance> {
		pub withdrawable: Balance,
		pub admin: AccountId,
		pub pending_admin: Option<AccountId>,
	}

	pub type OracleMetaOf<T> = OracleMeta<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

	/// Meta data tracking the oracle status for a feed.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct OracleStatus<Value> {
		pub starting_round: RoundId,
		pub ending_round: Option<RoundId>,
		pub last_reported_round: Option<RoundId>,
		pub last_started_round: Option<RoundId>,
		pub latest_submission: Option<Value>,
	}

	/// Minimum and Maximum number of submissions allowed per round.
	pub type SubmissionBounds = (u32, u32);

	pub type OracleStatusOf<T> = OracleStatus<<T as Config>::Value>;

	impl<Value> OracleStatus<Value>
	where
		Value: Default,
	{
		/// Create a new oracle status with the given `starting_round`.
		fn new(starting_round: RoundId) -> Self {
			Self {
				starting_round,
				..Default::default()
			}
		}
	}

	/// Used to store round requester permissions for accounts.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct Requester {
		pub delay: RoundId,
		pub last_started_round: Option<RoundId>,
	}

	/// Round data as served by the `FeedInterface`.
	#[derive(Clone, Encode, Decode, Default, Eq, PartialEq, RuntimeDebug)]
	pub struct RoundData<BlockNumber, Value> {
		pub started_at: BlockNumber,
		pub answer: Value,
		pub updated_at: BlockNumber,
		pub answered_in_round: RoundId,
	}

	pub type RoundDataOf<T> =
		RoundData<<T as frame_system::Config>::BlockNumber, <T as Config>::Value>;

	/// Possible error when converting from `Round` to `RoundData`.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
	pub enum RoundConversionError {
		MissingField,
	}

	// Implements a conversion from `Round` to `RoundData` so answered rounds can be converted easily.
	impl<B, V> TryFrom<Round<B, V>> for RoundData<B, V> {
		type Error = RoundConversionError;

		fn try_from(r: Round<B, V>) -> Result<Self, Self::Error> {
			if r.answered_in_round.is_none() || r.answer.is_none() || r.updated_at.is_none() {
				return Err(RoundConversionError::MissingField);
			}
			Ok(Self {
				started_at: r.started_at,
				answer: r.answer.unwrap(),
				updated_at: r.updated_at.unwrap(),
				answered_in_round: r.answered_in_round.unwrap(),
			})
		}
	}

	impl<B, V> RoundData<B, V> {
		/// Hard to use `Into` trait directly due to:
		/// https://doc.rust-lang.org/reference/items/traits.html#object-safety
		fn into_round(self) -> Round<B, V> {
			Round {
				started_at: self.started_at,
				answer: Some(self.answer),
				updated_at: Some(self.updated_at),
				answered_in_round: Some(self.answered_in_round),
			}
		}
	}

	/// Trait for interacting with the feeds in the pallet.
	pub trait FeedOracle<T: frame_system::Config> {
		type FeedId: Parameter + BaseArithmetic;
		type Feed: FeedInterface<T>;
		type MutableFeed: MutableFeedInterface<T>;

		/// Return the read-only interface for the given feed.
		///
		/// Returns `None` if the feed does not exist.
		fn feed(id: Self::FeedId) -> Option<Self::Feed>;

		/// Return the read-write interface for the given feed.
		///
		/// Returns `None` if the feed does not exist.
		fn feed_mut(id: Self::FeedId) -> Option<Self::MutableFeed>;
	}

	/// Trait for read-only access to a feed.
	pub trait FeedInterface<T: frame_system::Config> {
		type Value: Parameter + BaseArithmetic;

		/// Returns the id of the first round that contains non-default data.
		///
		/// Check this if you want to make sure that the data returned by `latest_data` is sensible.
		fn first_valid_round(&self) -> Option<RoundId>;

		/// Returns the id of the latest oracle round.
		fn latest_round(&self) -> RoundId;

		/// Returns the data for a given round.
		///
		/// Will return `None` if there is no data for the given round.
		fn data_at(&self, round: RoundId) -> Option<RoundData<T::BlockNumber, Self::Value>>;

		/// Returns the latest data for the feed.
		///
		/// Will always return data but may contain default data if there has not
		/// been a valid round, yet.
		/// Check `first_valid_round` to determine whether there is useful data, yet.
		fn latest_data(&self) -> RoundData<T::BlockNumber, Self::Value>;

		/// Represents the number of decimals with which the feed is configured
		fn decimals(&self) -> u8;
	}

	/// Trait for read-write access to a feed.
	pub trait MutableFeedInterface<T: frame_system::Config>: FeedInterface<T> {
		/// Request that a new oracle round be started.
		///
		/// **Warning:** Fallible function that changes storage.
		fn request_new_round(&mut self, requester: T::AccountId) -> DispatchResult;
	}

	#[pallet::config]
	#[allow(clippy::unused_unit)]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Type for feed indexing.
		type FeedId: Member + Parameter + Default + Copy + HasCompact + BaseArithmetic;

		/// Oracle feed values.
		type Value: Member + Parameter + Default + Copy + HasCompact + PartialEq + BaseArithmetic;

		/// Interface used for balance transfers.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The module id used to determine the account for storing the funds used to pay the oracles.
		type PalletId: Get<PalletId>;

		/// The minimum amount of funds that need to be present in the fund account.
		type MinimumReserve: Get<BalanceOf<Self>>;

		/// Maximum allowed string length.
		type StringLimit: Get<u32>;

		/// Maximum number of oracles per feed.
		type OracleCountLimit: Get<u32>;

		/// Maximum number of feeds.
		type FeedLimit: Get<Self::FeedId>;

		/// This callback will trigger when the round answer updates
		type OnAnswerHandler: OnAnswerHandler<Self>;

		/// The weight for this pallet's extrinsics.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn pallet_admin)]
	/// The account controlling the funds for this pallet.
	pub type PalletAdmin<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	// possible optimization: put together with admin?
	/// The account to set as future pallet admin.
	pub type PendingPalletAdmin<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn feed_counter)]
	/// A running counter used internally to determine the next feed id.
	pub type FeedCounter<T: Config> = StorageValue<_, T::FeedId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn feed_config)]
	/// A running counter used internally to determine the next feed id.
	pub type Feeds<T: Config> =
		StorageMap<_, Twox64Concat, T::FeedId, FeedConfigOf<T>, OptionQuery>;

	#[pallet::storage]
	/// Accounts allowed to create feeds.
	pub type FeedCreators<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// User-facing round data.
	pub type Rounds<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::FeedId,
		Twox64Concat,
		RoundId,
		RoundOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn round_details)]
	/// Operator-facing round data.
	pub type Details<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::FeedId,
		Twox64Concat,
		RoundId,
		RoundDetailsOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn oracle)]
	/// Global oracle meta data including admin and withdrawable funds.
	pub type Oracles<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, OracleMetaOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn oracle_status)]
	/// Feed local oracle status data.
	pub type OracleStatuses<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::FeedId,
		Blake2_128Concat,
		T::AccountId,
		OracleStatusOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn requester)]
	/// Per-feed permissioning for starting new rounds.
	pub type Requesters<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::FeedId,
		Blake2_128Concat,
		T::AccountId,
		Requester,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(
		T::AccountId = "AccountId",
		T::FeedId = "FeedId",
		T::BlockNumber = "BlockNumber",
		T::Value = "Value",
		RoundId = "RoundId",
		SubmissionBounds = "SubmissionBounds"
	)]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new oracle feed was created. \[feed_id, creator\]
		FeedCreated(T::FeedId, T::AccountId),
		/// A new round was started. \[new_round_id, initiator, started_at\]
		NewRound(T::FeedId, RoundId, T::AccountId, T::BlockNumber),
		/// A submission was recorded. \[feed_id, round_id, submission, oracle\]
		SubmissionReceived(T::FeedId, RoundId, T::Value, T::AccountId),
		/// The answer for the round was updated. \[feed_id, round_id, new_answer, updated_at_block\]
		AnswerUpdated(T::FeedId, RoundId, T::Value, T::BlockNumber),
		/// The round details were updated. \[feed_id, payment, submission_count_bounds, restart_delay, timeout\]
		RoundDetailsUpdated(
			T::FeedId,
			BalanceOf<T>,
			SubmissionBounds,
			RoundId,
			T::BlockNumber,
		),
		/// An admin change was requested for the given oracle. \[oracle, admin, pending_admin\]
		OracleAdminUpdateRequested(T::AccountId, T::AccountId, T::AccountId),
		/// The admin change was executed. \[oracle, new_admin\]
		OracleAdminUpdated(T::AccountId, T::AccountId),
		/// The submission permissions for the given feed and oracle have been updated. \[feed, oracle, enabled\]
		OraclePermissionsUpdated(T::FeedId, T::AccountId, bool),
		/// The requester permissions have been updated (set or removed). \[feed, requester, authorized, delays\]
		RequesterPermissionsSet(T::FeedId, T::AccountId, bool, RoundId),
		/// An owner change was requested for the given feed. \[feed, old_owner, new_owner\]
		OwnerUpdateRequested(T::FeedId, T::AccountId, T::AccountId),
		/// The owner change was executed. \[feed, new_owner\]
		OwnerUpdated(T::FeedId, T::AccountId),
		/// A pallet admin change was requested. \[old_pallet_admin, new_pallet_admin\]
		PalletAdminUpdateRequested(T::AccountId, T::AccountId),
		/// The pallet admin change was executed. \[new_admin\]
		PalletAdminUpdated(T::AccountId),
		/// The account is allowed to create feeds. \[new_creator\]
		FeedCreator(T::AccountId),
		/// The account is no longer allowed to create feeds. \[previously_creator\]
		FeedCreatorRemoved(T::AccountId),
		#[cfg(test)]
		/// New round data
		///
		/// Note:
		///
		/// This is only for tests
		NewData(T::FeedId, RoundData<T::BlockNumber, T::Value>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// A math operation lead to an overflow.
		Overflow,
		/// Given account id is not an oracle
		NotOracle,
		/// The oracle cannot submit as it is not enabled yet.
		OracleNotEnabled,
		/// The oracle has an ending round lower than the current round.
		OracleDisabled,
		/// More oracles were passed for disabling than are present.
		NotEnoughOracles,
		/// The oracle cannot report for past rounds.
		ReportingOrder,
		/// Requested feed not present.
		FeedNotFound,
		/// Requested round not present.
		RoundNotFound,
		/// The specified account does not have requester permissions stored.
		RequesterNotFound,
		/// New round cannot be requested to supersede current round.
		RoundNotSupersedable,
		/// No oracle meta data found for the given account.
		OracleNotFound,
		/// Submissions are not accepted for the specified round.
		NotAcceptingSubmissions,
		/// Oracle submission is below the minimum value.
		SubmissionBelowMinimum,
		/// Oracle submission is above the maximum value.
		SubmissionAboveMaximum,
		/// The description string is too long.
		DescriptionTooLong,
		/// Tried to add too many oracles.
		OraclesLimitExceeded,
		/// The oracle was already enabled.
		AlreadyEnabled,
		/// The oracle address cannot change its associated admin.
		OwnerCannotChangeAdmin,
		/// Only the owner of a feed can change the configuration.
		NotFeedOwner,
		/// Only the pending owner of a feed can accept the transfer invitation.
		NotPendingOwner,
		/// The specified min/max pair was invalid.
		WrongBounds,
		/// The maximum number of oracles cannot exceed the amount of available oracles.
		MaxExceededTotal,
		/// The round initiation delay cannot be equal to or greater
		/// than the number of oracles.
		DelayNotBelowCount,
		/// Sender is not admin. Admin privilege can only be transferred by the admin.
		NotAdmin,
		/// Only the pending admin can accept the transfer.
		NotPendingAdmin,
		/// The requester cannot request a new round, yet.
		CannotRequestRoundYet,
		/// No requester permissions associated with the given account.
		NotAuthorizedRequester,
		/// Cannot withdraw funds.
		InsufficientFunds,
		/// Funds cannot be withdrawn as the reserve would be critically low.
		InsufficientReserve,
		/// Only the pallet admin account can call this extrinsic.
		NotPalletAdmin,
		/// Only the pending admin can accept the transfer.
		NotPendingPalletAdmin,
		/// Round zero is not allowed to be pruned.
		CannotPruneRoundZero,
		/// The maximum number of feeds was reached.
		FeedLimitReached,
		/// The round cannot be superseded by a new round.
		NotSupersedable,
		/// The round cannot be started because it is not a valid new round.
		InvalidRound,
		/// The calling account is not allowed to create feeds.
		NotFeedCreator,
		/// The maximum debt of feeds was reached.
		MaxDebtReached,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	impl<T: Config> Pallet<T> {
		/// Shortcut for getting account ID
		fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}

		/// Get debt by FeedId
		pub fn debt(feed_id: T::FeedId) -> Result<BalanceOf<T>, Error<T>> {
			if let Some(feed_config) = <Feeds<T>>::get(feed_id) {
				Ok(feed_config.debt)
			} else {
				Err(<Error<T>>::FeedNotFound)
			}
		}

		fn genesis_feeds(
			owner: T::AccountId,
			payment: BalanceOf<T>,
			timeout: T::BlockNumber,
			submission_value_bounds: (T::Value, T::Value),
			min_submissions: u32,
			decimals: u8,
			description: Vec<u8>,
			restart_delay: RoundId,
			oracles: Vec<(T::AccountId, T::AccountId)>,
			pruning_window: RoundId,
			max_debt: Option<BalanceOf<T>>,
		) -> Result<(), Error<T>> {
			let submission_count_bounds = (min_submissions, oracles.len() as u32);
			let id: T::FeedId = FeedCounter::<T>::get();
			let new_id = id.add(One::one());
			FeedCounter::<T>::put(new_id);

			let new_config = FeedConfig {
				owner: owner.clone(),
				pending_owner: None,
				payment,
				timeout,
				submission_value_bounds,
				submission_count_bounds,
				decimals,
				description,
				restart_delay,
				latest_round: Zero::zero(),
				reporting_round: Zero::zero(),
				first_valid_round: None,
				oracle_count: oracles.len() as u32,
				pruning_window,
				next_round_to_prune: RoundId::one(),
				debt: Zero::zero(),
				max_debt,
			};

			let feed = Feed::<T>::new(id, new_config); // synced on drop
			let started_at = frame_system::Pallet::<T>::block_number();
			let updated_at = Some(started_at);
			// Store a dummy value for round 0 because we will not get useful data for
			// it, but need some seed data that future rounds can carry over.
			Rounds::<T>::insert(
				id,
				RoundId::zero(),
				Round {
					started_at,
					answer: Some(Zero::zero()),
					updated_at,
					answered_in_round: Some(Zero::zero()),
				},
			);

			for (oracle, admin) in oracles {
				Oracles::<T>::insert(
					&oracle,
					OracleMeta {
							withdrawable: Zero::zero(),
							admin,
							..Default::default()
						},
				);
				OracleStatuses::<T>::try_mutate(
					id,
					&oracle,
					|maybe_status| -> DispatchResult {
						// Only allow enabling non-existent or disabled oracles
						// in order to keep the count accurate.
						ensure!(
							maybe_status
								.as_ref()
								.map(|s| s.ending_round.is_some())
								.unwrap_or(true),
							Error::<T>::AlreadyEnabled
						);
						if let Some(status) = maybe_status.as_mut() {
							// overwrite the starting and ending round
							status.starting_round = feed.reporting_round_id();
							status.ending_round = None;
						} else {
							*maybe_status = Some(OracleStatus::new(feed.reporting_round_id()));
						}
						Ok(())
					},
				).unwrap();
			}
			Ok(())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// --- feed operations ---

		/// Create a new oracle feed with the given config values.
		/// Limited to feed creator accounts.
		#[pallet::weight(T::WeightInfo::create_feed(oracles.len() as u32))]
		#[allow(clippy::too_many_arguments)]
		pub fn create_feed(
			origin: OriginFor<T>,
			payment: BalanceOf<T>,
			timeout: T::BlockNumber,
			submission_value_bounds: (T::Value, T::Value),
			min_submissions: u32,
			decimals: u8,
			description: Vec<u8>,
			restart_delay: RoundId,
			oracles: Vec<(T::AccountId, T::AccountId)>,
			pruning_window: Option<RoundId>,
			max_debt: Option<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			ensure!(
				FeedCreators::<T>::contains_key(&owner),
				Error::<T>::NotFeedCreator
			);
			ensure!(
				description.len() as u32 <= T::StringLimit::get(),
				Error::<T>::DescriptionTooLong
			);

			let pruning_window = pruning_window.unwrap_or(RoundId::MAX);
			ensure!(
				pruning_window > RoundId::zero(),
				Error::<T>::CannotPruneRoundZero
			);

			let submission_count_bounds = (min_submissions, oracles.len() as u32);

			with_transaction_result(|| -> DispatchResultWithPostInfo {
				let id: T::FeedId = FeedCounter::<T>::get();
				ensure!(id < T::FeedLimit::get(), Error::<T>::FeedLimitReached);
				let new_id = id.checked_add(&One::one()).ok_or(Error::<T>::Overflow)?;
				FeedCounter::<T>::put(new_id);

				let new_config = FeedConfig {
					owner: owner.clone(),
					pending_owner: None,
					payment,
					timeout,
					submission_value_bounds,
					submission_count_bounds,
					decimals,
					description,
					restart_delay,
					latest_round: Zero::zero(),
					reporting_round: Zero::zero(),
					first_valid_round: None,
					oracle_count: Zero::zero(),
					pruning_window,
					next_round_to_prune: RoundId::one(),
					debt: Zero::zero(),
					max_debt,
				};
				let mut feed = Feed::<T>::new(id, new_config); // synced on drop
				let started_at = frame_system::Pallet::<T>::block_number();
				let updated_at = Some(started_at);
				// Store a dummy value for round 0 because we will not get useful data for
				// it, but need some seed data that future rounds can carry over.
				Rounds::<T>::insert(
					id,
					RoundId::zero(),
					Round {
						started_at,
						answer: Some(Zero::zero()),
						updated_at,
						answered_in_round: Some(Zero::zero()),
					},
				);
				feed.add_oracles(oracles)?;
				// validate the rounds config
				feed.update_future_rounds(
					payment,
					submission_count_bounds,
					restart_delay,
					timeout,
				)?;
				Self::deposit_event(Event::FeedCreated(id, owner));
				Ok(().into())
			})
		}

		/// Initiate the transfer of the feed to `new_owner`.
		#[pallet::weight(T::WeightInfo::transfer_ownership())]
		pub fn transfer_ownership(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			new_owner: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let old_owner = ensure_signed(origin)?;
			let mut feed = Self::feed_config(feed_id).ok_or(Error::<T>::FeedNotFound)?;
			ensure!(feed.owner == old_owner, Error::<T>::NotFeedOwner);

			feed.pending_owner = Some(new_owner.clone());
			Feeds::<T>::insert(feed_id, feed);

			Self::deposit_event(Event::OwnerUpdateRequested(feed_id, old_owner, new_owner));

			Ok(().into())
		}

		/// Accept the transfer of feed ownership.
		#[pallet::weight(T::WeightInfo::accept_ownership())]
		pub fn accept_ownership(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
		) -> DispatchResultWithPostInfo {
			let new_owner = ensure_signed(origin)?;
			let mut feed = Self::feed_config(feed_id).ok_or(Error::<T>::FeedNotFound)?;

			ensure!(
				feed.pending_owner.filter(|p| p == &new_owner).is_some(),
				Error::<T>::NotPendingOwner
			);

			feed.pending_owner = None;
			feed.owner = new_owner.clone();
			Feeds::<T>::insert(feed_id, feed);

			Self::deposit_event(Event::OwnerUpdated(feed_id, new_owner));

			Ok(().into())
		}

		/// Updates the pruning window of an existing feed
		///
		/// - Will prune rounds if the given window is smaller than the existing one.
		#[pallet::weight(1_000)]
		pub fn set_pruning_window(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			pruning_window: RoundId,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			ensure!(
				pruning_window > RoundId::zero(),
				Error::<T>::CannotPruneRoundZero
			);

			let mut feed = Feed::<T>::load_from(feed_id).ok_or(Error::<T>::FeedNotFound)?;
			feed.ensure_owner(&owner)?;

			feed.config.pruning_window = pruning_window;
			loop {
				// prune all rounds outside the window
				if !feed.prune_oldest() {
					break;
				}
			}

			Ok(().into())
		}

		/// Submit a new value to the given feed and round.
		///
		/// - Will start a new round if there is no round for the id, yet,
		///   and a round can be started (at this time by this oracle).
		/// - Will update the round answer if minimum number of submissions
		///   has been reached.
		/// - Records the rewards incurred by the oracle.
		/// - Removes the details for the previous round if it was superseded.
		///
		/// Limited to the oracles of a feed.
		#[pallet::weight(T::WeightInfo::submit_opening_round_answers().max(
		T::WeightInfo::submit_closing_answer(T::OracleCountLimit::get())
		))]
		pub fn submit(
			origin: OriginFor<T>,
			#[pallet::compact] feed_id: T::FeedId,
			#[pallet::compact] round_id: RoundId,
			#[pallet::compact] submission: T::Value,
		) -> DispatchResultWithPostInfo {
			let oracle = ensure_signed(origin)?;

			with_transaction_result(|| -> DispatchResultWithPostInfo {
				let mut feed = Feed::<T>::load_from(feed_id).ok_or(Error::<T>::FeedNotFound)?;
				let mut oracle_status =
					Self::oracle_status(feed_id, &oracle).ok_or(Error::<T>::NotOracle)?;
				feed.ensure_valid_round(&oracle, round_id)?;

				let (min_val, max_val) = feed.config.submission_value_bounds;
				ensure!(submission >= min_val, Error::<T>::SubmissionBelowMinimum);
				ensure!(submission <= max_val, Error::<T>::SubmissionAboveMaximum);

				let new_round_id = feed.reporting_round_id().saturating_add(One::one());
				let next_eligible_round = oracle_status
					.last_started_round
					.unwrap_or_else(Zero::zero)
					.checked_add(feed.config.restart_delay)
					.ok_or(Error::<T>::Overflow)?
					.checked_add(One::one())
					.ok_or(Error::<T>::Overflow)?;
				let eligible_to_start =
					round_id >= next_eligible_round || oracle_status.last_started_round.is_none();

				// initialize the round if conditions are met
				if round_id == new_round_id && eligible_to_start {
					let started_at = feed.initialize_round(new_round_id)?;

					Self::deposit_event(Event::NewRound(
						feed_id,
						new_round_id,
						oracle.clone(),
						started_at,
					));

					oracle_status.last_started_round = Some(new_round_id);
				}

				// record submission
				let mut details = Details::<T>::take(feed_id, round_id)
					.ok_or(Error::<T>::NotAcceptingSubmissions)?;
				details.submissions.push(submission);

				oracle_status.last_reported_round = Some(round_id);
				oracle_status.latest_submission = Some(submission);
				OracleStatuses::<T>::insert(feed_id, &oracle, oracle_status);
				Self::deposit_event(Event::SubmissionReceived(
					feed_id,
					round_id,
					submission,
					oracle.clone(),
				));

				// update round answer
				let (min_count, max_count) = details.submission_count_bounds;
				if details.submissions.len() >= min_count as usize {
					let updated_at = frame_system::Pallet::<T>::block_number();
					let new_answer = median(&mut details.submissions);
					let round = RoundData {
						started_at: Self::round(feed_id, round_id)
							.ok_or(Error::<T>::RoundNotFound)?
							.started_at,
						answer: new_answer,
						updated_at,
						answered_in_round: round_id,
					};

					Rounds::<T>::insert(feed_id, round_id, round.clone().into_round());

					feed.config.latest_round = round_id;
					if feed.config.first_valid_round.is_none() {
						feed.config.first_valid_round = Some(round_id);
					}
					// the previous rounds is not eligible for answers any more, so we close it
					let prev_round_id = round_id.saturating_sub(1);
					if prev_round_id > 0 {
						Details::<T>::remove(feed_id, prev_round_id);
					}
					// prune the oldest round
					feed.prune_oldest();

					T::OnAnswerHandler::on_answer(feed_id, round);
					Self::deposit_event(Event::AnswerUpdated(
						feed_id, round_id, new_answer, updated_at,
					));
				}

				// update oracle rewards and try to reserve them
				let payment = details.payment;
				// track the debt in case we cannot reserve
				T::Currency::reserve(&Self::account_id(), payment).or_else(
					|_| -> DispatchResult {
						// track the debt in case we cannot reserve
						let mut new_debt = feed.config.debt;
						new_debt = new_debt.checked_add(&payment).ok_or(Error::<T>::Overflow)?;

						if let Some(max_debt) = feed.config.max_debt {
							ensure!(new_debt < max_debt, <Error<T>>::MaxDebtReached);
						}

						feed.config.debt = new_debt;
						Ok(())
					},
				)?;

				let mut oracle_meta = Self::oracle(&oracle).ok_or(Error::<T>::OracleNotFound)?;
				oracle_meta.withdrawable = oracle_meta
					.withdrawable
					.checked_add(&payment)
					.ok_or(Error::<T>::Overflow)?;
				Oracles::<T>::insert(&oracle, oracle_meta);

				// delete the details if the maximum count has been reached
				if details.submissions.len() < max_count as usize {
					Details::<T>::insert(feed_id, round_id, details);
				}

				Ok(().into())
			})
		}

		/// Disable and add oracles for the given feed.
		/// Limited to the owner of a feed.
		#[pallet::weight(T::WeightInfo::change_oracles(to_disable.len() as u32, to_add.len() as u32))]
		pub fn change_oracles(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			to_disable: Vec<T::AccountId>,
			to_add: Vec<(T::AccountId, T::AccountId)>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;

			with_transaction_result(|| -> DispatchResultWithPostInfo {
				// synced on drop
				let mut feed = Feed::<T>::load_from(feed_id).ok_or(Error::<T>::FeedNotFound)?;
				feed.ensure_owner(&owner)?;
				feed.disable_oracles(to_disable)?;
				feed.add_oracles(to_add)?;

				Ok(().into())
			})
		}

		/// Update the configuration for future oracle rounds.
		/// Limited to the owner of a feed.
		#[pallet::weight(T::WeightInfo::update_future_rounds())]
		pub fn update_future_rounds(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			payment: BalanceOf<T>,
			submission_count_bounds: (u32, u32),
			restart_delay: RoundId,
			timeout: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			with_transaction_result(|| {
				// synced on drop
				let mut feed = Feed::<T>::load_from(feed_id).ok_or(Error::<T>::FeedNotFound)?;
				feed.ensure_owner(&owner)?;

				feed.update_future_rounds(
					payment,
					submission_count_bounds,
					restart_delay,
					timeout,
				)?;

				Ok(().into())
			})
		}

		// --- feed: round requests ---

		/// Set requester permissions for `requester`.
		/// Limited to the feed owner.
		#[pallet::weight(T::WeightInfo::set_requester())]
		pub fn set_requester(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			requester: T::AccountId,
			delay: RoundId,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			let feed = Self::feed_config(feed_id).ok_or(Error::<T>::FeedNotFound)?;
			ensure!(feed.owner == owner, Error::<T>::NotFeedOwner);

			// Keep the `last_started_round` if the requester already existed.
			let mut requester_meta = Self::requester(feed_id, &requester).unwrap_or_default();
			requester_meta.delay = delay;
			Requesters::<T>::insert(feed_id, &requester, requester_meta);

			Self::deposit_event(Event::RequesterPermissionsSet(
				feed_id, requester, true, delay,
			));

			Ok(().into())
		}

		/// Remove requester permissions for `requester`.
		/// Limited to the feed owner.
		#[pallet::weight(T::WeightInfo::remove_requester())]
		pub fn remove_requester(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			requester: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin)?;
			let feed = Self::feed_config(feed_id).ok_or(Error::<T>::FeedNotFound)?;
			ensure!(feed.owner == owner, Error::<T>::NotFeedOwner);

			let requester_meta =
				Requesters::<T>::take(feed_id, &requester).ok_or(Error::<T>::RequesterNotFound)?;

			Self::deposit_event(Event::RequesterPermissionsSet(
				feed_id,
				requester,
				false,
				requester_meta.delay,
			));

			Ok(().into())
		}

		/// Request the start of a new oracle round.
		/// Limited to accounts with "requester" permission.
		#[pallet::weight(T::WeightInfo::request_new_round())]
		pub fn request_new_round(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let mut requester =
				Self::requester(feed_id, &sender).ok_or(Error::<T>::NotAuthorizedRequester)?;

			with_transaction_result(|| -> DispatchResultWithPostInfo {
				let mut feed = Feed::<T>::load_from(feed_id).ok_or(Error::<T>::FeedNotFound)?;

				let new_round = feed
					.reporting_round_id()
					.checked_add(One::one())
					.ok_or(Error::<T>::Overflow)?;
				let last_started = requester.last_started_round.unwrap_or_else(Zero::zero);
				let next_allowed_round = last_started
					.checked_add(requester.delay)
					.ok_or(Error::<T>::Overflow)?;
				ensure!(
					requester.last_started_round.is_none() || new_round > next_allowed_round,
					Error::<T>::CannotRequestRoundYet
				);

				requester.last_started_round = Some(new_round);
				Requesters::<T>::insert(feed_id, &sender, requester);

				feed.request_new_round(sender)?;

				Ok(().into())
			})
		}

		// --- oracle operations ---

		/// Withdraw `amount` payment of the given oracle to `recipient`.
		/// Limited to the oracle admin.
		#[pallet::weight(T::WeightInfo::withdraw_payment())]
		pub fn withdraw_payment(
			origin: OriginFor<T>,
			oracle: T::AccountId,
			recipient: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let admin = ensure_signed(origin)?;
			let mut oracle_meta = Self::oracle(&oracle).ok_or(Error::<T>::OracleNotFound)?;
			ensure!(oracle_meta.admin == admin, Error::<T>::NotAdmin);

			oracle_meta.withdrawable = oracle_meta
				.withdrawable
				.checked_sub(&amount)
				.ok_or(Error::<T>::InsufficientFunds)?;

			T::Currency::transfer(
				&T::PalletId::get().into_account(),
				&recipient,
				amount,
				ExistenceRequirement::KeepAlive,
			)?;
			Oracles::<T>::insert(&oracle, oracle_meta);

			Ok(().into())
		}

		/// Initiate an admin transfer for the given oracle.
		/// Limited to the oracle admin account.
		#[pallet::weight(T::WeightInfo::transfer_admin())]
		pub fn transfer_admin(
			origin: OriginFor<T>,
			oracle: T::AccountId,
			new_admin: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let old_admin = ensure_signed(origin)?;
			let mut oracle_meta = Self::oracle(&oracle).ok_or(Error::<T>::OracleNotFound)?;

			ensure!(oracle_meta.admin == old_admin, Error::<T>::NotAdmin);

			oracle_meta.pending_admin = Some(new_admin.clone());
			Oracles::<T>::insert(&oracle, oracle_meta);

			Self::deposit_event(Event::OracleAdminUpdateRequested(
				oracle, old_admin, new_admin,
			));

			Ok(().into())
		}

		/// Complete an admin transfer for the given oracle.
		/// Limited to the pending oracle admin account.
		#[pallet::weight(T::WeightInfo::accept_admin())]
		pub fn accept_admin(
			origin: OriginFor<T>,
			oracle: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let new_admin = ensure_signed(origin)?;
			let mut oracle_meta = Self::oracle(&oracle).ok_or(Error::<T>::OracleNotFound)?;

			ensure!(
				oracle_meta
					.pending_admin
					.filter(|p| p == &new_admin)
					.is_some(),
				Error::<T>::NotPendingAdmin
			);

			oracle_meta.pending_admin = None;
			oracle_meta.admin = new_admin.clone();
			Oracles::<T>::insert(&oracle, oracle_meta);

			Self::deposit_event(Event::OracleAdminUpdated(oracle, new_admin));

			Ok(().into())
		}

		// --- pallet admin operations ---

		/// Withdraw `amount` funds to `recipient`.
		/// Limited to the pallet admin.
		#[pallet::weight(T::WeightInfo::withdraw_funds())]
		pub fn withdraw_funds(
			origin: OriginFor<T>,
			recipient: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::pallet_admin(), Error::<T>::NotPalletAdmin);
			let fund = T::PalletId::get().into_account();
			let reserve = T::Currency::free_balance(&fund);
			let new_reserve = reserve
				.checked_sub(&amount)
				.ok_or(Error::<T>::InsufficientFunds)?;
			ensure!(
				new_reserve >= T::MinimumReserve::get(),
				Error::<T>::InsufficientReserve
			);
			T::Currency::transfer(&fund, &recipient, amount, ExistenceRequirement::KeepAlive)?;

			Ok(().into())
		}

		/// Reduce the amount of debt in the pallet by moving funds from
		/// the free balance to the reserved so oracles can be payed out.
		/// Limited to the pallet admin.
		#[pallet::weight(T::WeightInfo::reduce_debt())]
		pub fn reduce_debt(
			origin: OriginFor<T>,
			feed_id: T::FeedId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let _sender = ensure_signed(origin)?;
			let mut feed = <Feed<T>>::load_from(feed_id).ok_or(<Error<T>>::FeedNotFound)?;

			let to_reserve = amount.min(feed.config.debt);
			T::Currency::reserve(&Self::account_id(), to_reserve)?;
			// it's fine if we saturate to 0 debt
			feed.config.debt = feed.config.debt.saturating_sub(amount);

			Ok(().into())
		}

		/// Initiate an admin transfer for the pallet.
		/// Limited to the pallet admin account.
		#[pallet::weight(T::WeightInfo::transfer_pallet_admin())]
		pub fn transfer_pallet_admin(
			origin: OriginFor<T>,
			new_pallet_admin: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let old_admin = ensure_signed(origin)?;

			ensure!(
				Self::pallet_admin() == old_admin,
				Error::<T>::NotPalletAdmin
			);

			PendingPalletAdmin::<T>::put(&new_pallet_admin);

			Self::deposit_event(Event::PalletAdminUpdateRequested(
				old_admin,
				new_pallet_admin,
			));

			Ok(().into())
		}

		/// Complete an admin transfer for the pallet.
		/// Limited to the pending pallet admin account.
		#[pallet::weight(T::WeightInfo::accept_pallet_admin())]
		pub fn accept_pallet_admin(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let new_pallet_admin = ensure_signed(origin)?;

			ensure!(
				PendingPalletAdmin::<T>::get()
					.filter(|p| p == &new_pallet_admin)
					.is_some(),
				Error::<T>::NotPendingPalletAdmin
			);

			PendingPalletAdmin::<T>::take();
			PalletAdmin::<T>::put(&new_pallet_admin);

			Self::deposit_event(Event::PalletAdminUpdated(new_pallet_admin));

			Ok(().into())
		}

		/// Allow the given account to create oracle feeds.
		/// Limited to the pallet admin account.
		#[pallet::weight(T::WeightInfo::set_feed_creator())]
		pub fn set_feed_creator(
			origin: OriginFor<T>,
			new_creator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let admin = ensure_signed(origin)?;
			ensure!(Self::pallet_admin() == admin, Error::<T>::NotPalletAdmin);

			FeedCreators::<T>::insert(&new_creator, ());

			Self::deposit_event(Event::FeedCreator(new_creator));

			Ok(().into())
		}

		/// Disallow the given account to create oracle feeds.
		/// Limited to the pallet admin account.
		#[pallet::weight(T::WeightInfo::remove_feed_creator())]
		pub fn remove_feed_creator(
			origin: OriginFor<T>,
			creator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let admin = ensure_signed(origin)?;
			ensure!(Self::pallet_admin() == admin, Error::<T>::NotPalletAdmin);

			FeedCreators::<T>::remove(&creator);

			Self::deposit_event(Event::FeedCreatorRemoved(creator));

			Ok(().into())
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub pallet_admin: Option<T::AccountId>,
		// accounts configured at genesis to be allowed to create new feeds
		pub feed_creators: Vec<T::AccountId>,
		pub feeds: Vec<(T::AccountId, BalanceOf<T>, T::BlockNumber, u32, u8, Vec<u8>, Vec<(T::AccountId, T::AccountId)>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				pallet_admin: Default::default(),
				feed_creators: Default::default(),
				feeds: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			if let Some(ref admin) = self.pallet_admin {
				PalletAdmin::<T>::put(admin);
			}

			for creator in &self.feed_creators {
				FeedCreators::<T>::insert(creator, ());
			}

			for feed in &self.feeds {
				// force panic here
				Pallet::<T>::genesis_feeds(
					feed.0.clone(),
					feed.1.clone(),
					feed.2.clone(),
					(T::Value::zero(), T::Value::max_value()),
					feed.3.clone(),
					feed.4.clone(),
					feed.5.clone(),
					0,
					feed.6.clone(),
					10,
					None,
				).unwrap();
			}
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		/// Direct implementation of `GenesisBuild::build_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn build_storage(&self) -> Result<frame_support::sp_runtime::Storage, String> {
			<Self as GenesisBuild<T>>::build_storage(self)
		}

		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn assimilate_storage(
			&self,
			storage: &mut frame_support::sp_runtime::Storage,
		) -> Result<(), String> {
			<Self as GenesisBuild<T>>::assimilate_storage(self, storage)
		}
	}

	/// Proxy used for interaction with a feed.
	/// `should_sync` flag determines whether the `config` is put into
	/// storage on `drop`.
	pub struct Feed<T: Config> {
		pub id: T::FeedId,
		pub config: FeedConfigOf<T>,
		pub should_sync: bool,
	}

	impl<T: Config> Feed<T> {
		// --- constructors ---

		/// Create a new feed with the given id and config.
		/// Will store the config when dropped.
		pub fn new(id: T::FeedId, config: FeedConfigOf<T>) -> Self {
			Self {
				id,
				config,
				should_sync: true,
			}
		}

		/// Load the feed with the given id for reading.
		/// Will not store the config when dropped.
		/// -> Don't mutate the feed object.
		pub fn read_only_from(id: T::FeedId) -> Option<Self> {
			let config = Feeds::<T>::get(id)?;
			Some(Self {
				id,
				config,
				should_sync: false,
			})
		}

		/// Load the feed with the given id from storage.
		/// Will store the config when dropped.
		pub fn load_from(id: T::FeedId) -> Option<Self> {
			let config = Feeds::<T>::get(id)?;
			Some(Self {
				id,
				config,
				should_sync: true,
			})
		}

		// --- getters ---

		/// Return the round oracles are currently reporting data for.
		pub fn reporting_round_id(&self) -> RoundId {
			self.config.reporting_round
		}

		/// Return the round data (including the answer, if present).
		fn round(&self, round: RoundId) -> Option<RoundOf<T>> {
			Rounds::<T>::get(self.id, round)
		}

		/// Return the round details (including submissions).
		pub fn details(&self, round: RoundId) -> Option<RoundDetailsOf<T>> {
			Details::<T>::get(self.id, round)
		}

		/// Return the oracle status associated with this feed.
		fn status(&self, oracle: &T::AccountId) -> Option<OracleStatusOf<T>> {
			OracleStatuses::<T>::get(self.id, oracle)
		}

		/// Return the number of oracles that can submit data for this feed.
		fn oracle_count(&self) -> u32 {
			self.config.oracle_count
		}

		// --- checks ---

		/// Make sure that the given account is the owner of the feed.
		fn ensure_owner(&self, owner: &T::AccountId) -> DispatchResult {
			ensure!(&self.config.owner == owner, Error::<T>::NotFeedOwner);
			Ok(())
		}

		/// Make sure that the given oracle can submit data for the given round.
		fn ensure_valid_round(&self, oracle: &T::AccountId, round_id: RoundId) -> DispatchResult {
			let o = self.status(oracle).ok_or(Error::<T>::NotOracle)?;

			ensure!(o.starting_round <= round_id, Error::<T>::OracleNotEnabled);
			ensure!(
				o.ending_round.map(|e| e >= round_id).unwrap_or(true),
				Error::<T>::OracleDisabled
			);
			ensure!(
				o.last_reported_round.map(|l| l < round_id).unwrap_or(true),
				Error::<T>::ReportingOrder
			);
			let is_current = round_id == self.reporting_round_id();
			let is_next = round_id == self.reporting_round_id().saturating_add(One::one());
			let current_unanswered = self
				.round(self.reporting_round_id())
				.map(|r| r.updated_at.is_none())
				.unwrap_or(true);
			let is_previous = round_id.saturating_add(One::one()) == self.reporting_round_id();
			ensure!(
				is_current || is_next || (is_previous && current_unanswered),
				Error::<T>::InvalidRound
			);
			ensure!(
				round_id == RoundId::one()
					|| self.is_supersedable(round_id.saturating_sub(One::one())),
				Error::<T>::NotSupersedable
			);
			Ok(())
		}

		/// Check whether a round is timed out.
		/// Returns `false` for rounds not present in storage.
		fn is_timed_out(&self, round: RoundId) -> bool {
			// Assumption: returning false for non-existent rounds is fine.
			let started_at = self
				.round(round)
				.map(|r| r.started_at)
				.unwrap_or_else(Zero::zero);
			let timeout = self
				.details(round)
				.map(|d| d.timeout)
				.unwrap_or_else(Zero::zero);
			let block_num = frame_system::Pallet::<T>::block_number();

			started_at > Zero::zero()
				&& timeout > Zero::zero()
				&& started_at.saturating_add(timeout) < block_num
		}

		/// Check whether a round has been updated.
		/// Returns `false` for rounds not present in storage.
		fn was_updated(&self, round: RoundId) -> bool {
			self.round(round)
				.map(|r| r.updated_at.is_some())
				.unwrap_or(false)
		}

		/// Check whether the round can be superseded by the next one.
		/// Returns `false` for rounds not present in storage.
		fn is_supersedable(&self, round: RoundId) -> bool {
			round == RoundId::zero() || self.was_updated(round) || self.is_timed_out(round)
		}

		// --- mutators ---
		/// Add the given oracles to the feed.
		#[require_transactional]
		pub fn add_oracles(&mut self, to_add: Vec<(T::AccountId, T::AccountId)>) -> DispatchResult {
			let new_count = self
				.oracle_count()
				// saturating is fine because we enforce a limit below
				.saturating_add(to_add.len() as u32);
			ensure!(
				new_count <= T::OracleCountLimit::get(),
				Error::<T>::OraclesLimitExceeded
			);
			self.config.oracle_count = new_count;
			for (oracle, admin) in to_add {
				if let Some(meta) = Oracles::<T>::get(&oracle) {
					// Make sure the admin is correct in case the oracle
					// is already tracked.
					ensure!(meta.admin == admin, Error::<T>::OwnerCannotChangeAdmin);
				} else {
					// Initialize the oracle if it is not tracked, yet.
					Oracles::<T>::insert(
						&oracle,
						OracleMeta {
							withdrawable: Zero::zero(),
							admin,
							..Default::default()
						},
					);
				}
				OracleStatuses::<T>::try_mutate(
					self.id,
					&oracle,
					|maybe_status| -> DispatchResult {
						// Only allow enabling non-existent or disabled oracles
						// in order to keep the count accurate.
						ensure!(
							maybe_status
								.as_ref()
								.map(|s| s.ending_round.is_some())
								.unwrap_or(true),
							Error::<T>::AlreadyEnabled
						);
						if let Some(status) = maybe_status.as_mut() {
							// overwrite the starting and ending round
							status.starting_round = self.reporting_round_id();
							status.ending_round = None;
						} else {
							*maybe_status = Some(OracleStatus::new(self.reporting_round_id()));
						}
						Ok(())
					},
				)?;
				Pallet::<T>::deposit_event(Event::OraclePermissionsUpdated(self.id, oracle, true));
			}

			Ok(())
		}

		/// Disable the given oracles.
		#[require_transactional]
		fn disable_oracles(&mut self, to_disable: Vec<T::AccountId>) -> DispatchResult {
			let disabled_count = to_disable.len() as u32;
			self.config.oracle_count = self
				.config
				.oracle_count
				.checked_sub(disabled_count)
				.ok_or(Error::<T>::NotEnoughOracles)?;
			for d in to_disable {
				let mut status = self.status(&d).ok_or(Error::<T>::OracleNotFound)?;
				ensure!(status.ending_round.is_none(), Error::<T>::OracleDisabled);
				status.ending_round = Some(self.reporting_round_id());
				OracleStatuses::<T>::insert(self.id, &d, status);
				Pallet::<T>::deposit_event(Event::OraclePermissionsUpdated(self.id, d, false));
			}
			Ok(())
		}

		/// Update the configuration for future oracle rounds.
		/// (Past and present rounds are unaffected.)
		#[require_transactional]
		pub fn update_future_rounds(
			&mut self,
			payment: BalanceOf<T>,
			submission_count_bounds: (u32, u32),
			restart_delay: RoundId,
			timeout: T::BlockNumber,
		) -> DispatchResult {
			let (min, max) = submission_count_bounds;
			ensure!(max >= min, Error::<T>::WrongBounds);
			// Make sure that both the min and max of submissions is
			// less or equal to the number of oracles.
			ensure!(self.oracle_count() >= max, Error::<T>::MaxExceededTotal);
			// Make sure that at least one oracle can request a new
			// round.
			ensure!(
				self.oracle_count() > restart_delay,
				Error::<T>::DelayNotBelowCount
			);
			if self.oracle_count() > 0 {
				ensure!(min > 0, Error::<T>::WrongBounds);
			}

			self.config.payment = payment;
			self.config.submission_count_bounds = submission_count_bounds;
			self.config.restart_delay = restart_delay;
			self.config.timeout = timeout;

			Pallet::<T>::deposit_event(Event::RoundDetailsUpdated(
				self.id,
				payment,
				submission_count_bounds,
				restart_delay,
				timeout,
			));
			Ok(())
		}

		/// Prune the state of a feed to reduce storage load.
		///
		/// Returns `true` if round was pruned, `false otherwise`
		fn prune_oldest(&mut self) -> bool {
			let prune_next = self.config.next_round_to_prune;
			// only prune if window is exceeded
			if self.config.latest_round.saturating_sub(prune_next) >= self.config.pruning_window {
				Rounds::<T>::remove(self.id, prune_next);
				Details::<T>::remove(self.id, prune_next);
				// update oldest round
				self.config.next_round_to_prune += RoundId::one();
				self.config.first_valid_round = Some(self.config.next_round_to_prune);
				true
			} else {
				false
			}
		}

		/// Initialize a new round.
		/// Will close the previous one if it is timed out.
		/// Will prune the oldest round that is outside the pruning window
		///
		/// **Warning:** Fallible function that changes storage.
		#[require_transactional]
		fn initialize_round(
			&mut self,
			new_round_id: RoundId,
		) -> Result<T::BlockNumber, DispatchError> {
			self.config.reporting_round = new_round_id;

			let prev_round_id = new_round_id.saturating_sub(One::one());
			if self.is_timed_out(prev_round_id) {
				self.close_timed_out_round(prev_round_id)?;
			}

			Details::<T>::insert(
				self.id,
				new_round_id,
				RoundDetails {
					submissions: Vec::new(),
					submission_count_bounds: self.config.submission_count_bounds,
					payment: self.config.payment,
					timeout: self.config.timeout,
				},
			);
			let started_at = frame_system::Pallet::<T>::block_number();
			Rounds::<T>::insert(self.id, new_round_id, Round::new(started_at));

			Ok(started_at)
		}

		/// Close a timed out round and remove its details.
		#[require_transactional]
		fn close_timed_out_round(&self, timed_out_id: RoundId) -> DispatchResult {
			let prev_id = timed_out_id.saturating_sub(One::one());
			let prev_round = self.round(prev_id).ok_or(Error::<T>::RoundNotFound)?;
			let mut timed_out_round = self.round(timed_out_id).ok_or(Error::<T>::RoundNotFound)?;
			timed_out_round.answer = prev_round.answer;
			timed_out_round.answered_in_round = prev_round.answered_in_round;
			let updated_at = frame_system::Pallet::<T>::block_number();
			timed_out_round.updated_at = Some(updated_at);

			Rounds::<T>::insert(self.id, timed_out_id, timed_out_round);
			Details::<T>::remove(self.id, timed_out_id);

			Ok(())
		}

		/// Store the feed config in storage.
		fn sync_to_storage(&mut self) {
			Feeds::<T>::insert(self.id, sp_std::mem::take(&mut self.config));
		}
	}

	// We want the feed to sync automatically when going out of scope.
	impl<T: Config> Drop for Feed<T> {
		fn drop(&mut self) {
			if self.should_sync {
				self.sync_to_storage();
			}
		}
	}

	impl<T: Config> FeedOracle<T> for Pallet<T> {
		type FeedId = T::FeedId;
		type Feed = Feed<T>;
		type MutableFeed = Feed<T>;

		/// Return a transient feed proxy object for interacting with the feed given by the id.
		/// Provides read-only access.
		fn feed(id: Self::FeedId) -> Option<Self::Feed> {
			Feed::read_only_from(id)
		}

		/// Return a transient feed proxy object for interacting with the feed given by the id.
		/// Provides read-write access.
		fn feed_mut(id: Self::FeedId) -> Option<Self::MutableFeed> {
			Feed::load_from(id)
		}
	}

	impl<T: Config> FeedInterface<T> for Feed<T> {
		type Value = T::Value;

		/// Returns the id of the first round that contains non-default data.
		fn first_valid_round(&self) -> Option<RoundId> {
			self.config.first_valid_round
		}

		/// Returns the id of the latest oracle round.
		fn latest_round(&self) -> RoundId {
			self.config.latest_round
		}

		/// Returns the data for a given round.
		fn data_at(&self, round: RoundId) -> Option<RoundData<T::BlockNumber, T::Value>> {
			self.round(round)?.try_into().ok()
		}

		/// Returns the latest data for the feed.
		fn latest_data(&self) -> RoundData<T::BlockNumber, T::Value> {
			let latest_round = self.latest_round();
			self.data_at(latest_round).unwrap_or_else(|| {
				debug_assert!(false, "The latest round data should always be available.");
				RoundData::default()
			})
		}

		/// Returns the configured decimals
		fn decimals(&self) -> u8 {
			self.config.decimals
		}
	}

	impl<T: Config> MutableFeedInterface<T> for Feed<T> {
		/// Requests that a new round be started for the feed.
		///
		/// Returns `Ok` on success and `Err` in case the round could not be started.
		#[require_transactional]
		fn request_new_round(&mut self, requester: T::AccountId) -> DispatchResult {
			let new_round = self
				.reporting_round_id()
				.checked_add(One::one())
				.ok_or(Error::<T>::Overflow)?;
			ensure!(
				self.is_supersedable(self.reporting_round_id()),
				Error::<T>::RoundNotSupersedable
			);
			let started_at = self.initialize_round(new_round)?;

			Pallet::<T>::deposit_event(Event::NewRound(self.id, new_round, requester, started_at));

			Ok(())
		}
	}

	/// Trait for the chainlink pallet extrinsic weights.
	pub trait WeightInfo {
		fn create_feed(o: u32) -> Weight;
		fn transfer_ownership() -> Weight;
		fn accept_ownership() -> Weight;
		fn submit_opening_round_answers() -> Weight;
		fn submit_closing_answer(o: u32) -> Weight;
		fn change_oracles(d: u32, n: u32) -> Weight;
		fn update_future_rounds() -> Weight;
		fn set_requester() -> Weight;
		fn remove_requester() -> Weight;
		fn request_new_round() -> Weight;
		fn withdraw_payment() -> Weight;
		fn transfer_admin() -> Weight;
		fn accept_admin() -> Weight;
		fn withdraw_funds() -> Weight;
		fn reduce_debt() -> Weight;
		fn transfer_pallet_admin() -> Weight;
		fn accept_pallet_admin() -> Weight;
		fn set_feed_creator() -> Weight;
		fn remove_feed_creator() -> Weight;
	}
}
