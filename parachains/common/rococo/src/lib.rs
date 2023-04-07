#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use frame_support::traits::Contains;
use sp_core::Get;
use xcm::latest::prelude::*;

pub struct RelayOrOtherSystemParachains<Runtime: parachain_info::Config> {
	_runtime: PhantomData<Runtime>,
}
impl<Runtime: parachain_info::Config> Contains<MultiLocation>
	for RelayOrOtherSystemParachains<Runtime>
{
	fn contains(l: &MultiLocation) -> bool {
		let self_para_id: u32 = parachain_info::Pallet::<Runtime>::get().into();
		if let MultiLocation { parents: 0, interior: X1(Parachain(para_id)) } = l {
			if *para_id == self_para_id {
				return false
			}
		}
		matches!(l, MultiLocation { parents: 1, interior: Here }) ||
			rococo_runtime_constants::system_parachain::SystemParachains::contains(l)
	}
}
