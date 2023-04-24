pub use polkadot_runtime_parachains::configuration::HostConfiguration;
pub use parachains_common::BlockNumber;

pub mod accounts {
	pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
	pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);
}

pub mod polkadot {
	use super::*;

	pub fn get_host_config() -> HostConfiguration<BlockNumber> {
		HostConfiguration {
			max_upward_queue_count: 10,
			max_upward_queue_size: 51200,
			max_upward_message_size: 51200,
			max_upward_message_num_per_candidate: 10,
			max_downward_message_size: 51200,
			..Default::default()
		}
	}
}

pub mod kusama {
	use super::*;

	pub fn get_host_config() -> HostConfiguration<BlockNumber> {
		HostConfiguration {
			max_upward_queue_count: 10,
			max_upward_queue_size: 51200,
			max_upward_message_size: 51200,
			max_upward_message_num_per_candidate: 10,
			max_downward_message_size: 51200,
			..Default::default()
		}
	}
}

// pub const POLKADOT_HOST_CONFIG: HostConfiguration<BlockNumber> = HostConfiguration {
// 	max_upward_queue_count: 10,
// 	max_upward_queue_size: 51200,
// 	max_upward_message_size: 51200,
// 	max_upward_message_num_per_candidate: 10,
// 	max_downward_message_size: 51200,
// 	..ConstDefault::default()
// };

// pub let KUSAMA_HOST_CONFIG: HostConfiguration<BlockNumber> = HostConfiguration {
// 	max_upward_queue_count: 10,
// 	max_upward_queue_size: 51200,
// 	max_upward_message_size: 51200,
// 	max_upward_message_num_per_candidate: 10,
// 	max_downward_message_size: 51200,
// 	..Default::default()
// };

// pub const fn get_polkadot_host_config() -> HostConfiguration<BlockNumber> {
// 	HostConfiguration {
// 		max_upward_queue_count: 10,
// 		max_upward_queue_size: 51200,
// 		max_upward_message_size: 51200,
// 		max_upward_message_num_per_candidate: 10,
// 		max_downward_message_size: 51200,
// 		..Default::default()
// 	}
// }

// pub const fn get_kusama_host_config() -> HostConfiguration<BlockNumber> {
// 	HostConfiguration {
// 		max_upward_queue_count: 10,
// 		max_upward_queue_size: 51200,
// 		max_upward_message_size: 51200,
// 		max_upward_message_num_per_candidate: 10,
// 		max_downward_message_size: 51200,
// 		..Default::default()
// 	}
// }
