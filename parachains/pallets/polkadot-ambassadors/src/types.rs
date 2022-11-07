//! Types

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{ConstU32, Get},
	BoundedVec, CloneNoBound, EqNoBound, PartialEqNoBound,
};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{fmt::Debug, prelude::*};

/// A unique integer identifier for each region.
pub type RegionId = u16;
/// A string, presumably to hold a link to a program charter.
pub type CharterLocation<StringLimit> = BoundedVec<u8, StringLimit>;
/// A unique integer identifier for each announcement.
// TODO: Could be a hash?
pub type AnnouncementId = u32;

/// The rank of a member.
#[derive(
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	RuntimeDebug,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
)]
pub enum Rank {
	Applicant,
	Ambassador,
	SeniorAmbassador,
	HeadAmbassador,
}

/// A member's contact info. Must provide at least one.
#[derive(
	CloneNoBound, Encode, Decode, EqNoBound, PartialEqNoBound, Default, TypeInfo, MaxEncodedLen,
)]
#[codec(mel_bound())]
#[scale_info(skip_type_params(StringLimit))]
pub struct Contact<StringLimit: Get<u32>> {
	matrix: Option<BoundedVec<u8, StringLimit>>,
	discord: Option<BoundedVec<u8, StringLimit>>,
	telegram: Option<BoundedVec<u8, StringLimit>>,
	twitter: Option<BoundedVec<u8, StringLimit>>,
	email: Option<BoundedVec<u8, StringLimit>>,
}

#[cfg(feature = "std")]
impl<StringLimit: Get<u32>> Debug for Contact<StringLimit> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(
			f,
			"Matrix: {:?}, Discord: {:?}, Telegram: {:?}, Twitter: {:?}, Email: {:?}",
			self.matrix, self.discord, self.telegram, self.twitter, self.email
		)
	}
}

/// Information about a region.
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
pub struct Region<StringLimit: Get<u32>, AccountId> {
	pub name: BoundedVec<u8, StringLimit>,
	pub lead: Option<AccountId>,
}

#[cfg(feature = "std")]
impl<StringLimit: Get<u32>, AccountId: Debug> Debug for Region<StringLimit, AccountId> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "Region name: {:?}, Region lead: {:?}", self.name, self.lead)
	}
}

/// The types of activities that the ambassador program works on. Users can work on more than one.
#[derive(Clone, Copy, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ActivityTrack {
	Events,
	ContentCreation,
	Moderation,
}

/// Information about a member of the program.
#[derive(Clone, Encode, Decode, RuntimeDebug, MaxEncodedLen)]
pub struct MemberInfo<StringLimit: Get<u32>> {
	/// The rank.
	rank: Rank,
	/// Contact info.
	contact: Contact<StringLimit>,
	/// Region.
	region: RegionId,
	/// Activities the member contributes to.
	activity_tracks: BoundedVec<ActivityTrack, ConstU32<4>>,
	/// Whether this user accepts payment or not.
	paid: bool,
}

impl<StringLimit: Get<u32> + 'static> TypeInfo for MemberInfo<StringLimit> {
	type Identity = Self;
	fn type_info() -> scale_info::Type {
		scale_info::Type::builder()
			.path(scale_info::Path::new("MemberInfo", module_path!()))
			.composite(
				scale_info::build::Fields::unnamed().field(|f| f.ty::<u8>().docs(
					&["Raw member info byte, encodes rank, contact, region, activity tracks, and whether they are paid."]
				)),
			)
	}
}

/// An announcement from the program.
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(StringLimit))]
pub struct Announcement<StringLimit: Get<u32>> {
	/// Under what category does this announcement fall?
	track: ActivityTrack,
	/// A URL for where to find more info.
	link: BoundedVec<u8, StringLimit>,
}

#[cfg(feature = "std")]
impl<StringLimit: Get<u32>> Debug for Announcement<StringLimit> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		write!(f, "Ambassadors Announcement! Track: {:?}, Link: {:?}", self.track, self.link)
	}
}
