
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::weights::{Weight, constants::RocksDbWeight as DbWeight};

pub struct WeightInfo;
impl pallet_chainlink_feed::WeightInfo for WeightInfo {
    fn create_feed(o: u32, ) -> Weight {
        (305_438_000 as Weight)
            .saturating_add((62_577_000 as Weight).saturating_mul(o as Weight))
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(o as Weight)))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(o as Weight)))
    }
    fn transfer_ownership() -> Weight {
        (79_760_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn accept_ownership() -> Weight {
        (77_660_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn submit_opening_round_answers() -> Weight {
        (442_347_000 as Weight)
            .saturating_add(DbWeight::get().reads(7 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn submit_closing_answer(o: u32, ) -> Weight {
        (362_642_000 as Weight)
            .saturating_add((1_219_000 as Weight).saturating_mul(o as Weight))
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(7 as Weight))
    }
    fn change_oracles(d: u32, n: u32, ) -> Weight {
        (0 as Weight)
            .saturating_add((74_485_000 as Weight).saturating_mul(d as Weight))
            .saturating_add((85_010_000 as Weight).saturating_mul(n as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().reads((1 as Weight).saturating_mul(d as Weight)))
            .saturating_add(DbWeight::get().reads((2 as Weight).saturating_mul(n as Weight)))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((1 as Weight).saturating_mul(d as Weight)))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(n as Weight)))
    }
    fn update_future_rounds() -> Weight {
        (87_446_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn prune(r: u32, ) -> Weight {
        (64_989_000 as Weight)
            .saturating_add((21_359_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_add(DbWeight::get().writes((2 as Weight).saturating_mul(r as Weight)))
    }
    fn set_requester() -> Weight {
        (99_375_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_requester() -> Weight {
        (90_710_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn request_new_round() -> Weight {
        (245_256_000 as Weight)
            .saturating_add(DbWeight::get().reads(4 as Weight))
            .saturating_add(DbWeight::get().writes(4 as Weight))
    }
    fn withdraw_payment() -> Weight {
        (210_562_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
    fn transfer_admin() -> Weight {
        (78_330_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn accept_admin() -> Weight {
        (77_813_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn withdraw_funds() -> Weight {
        (183_123_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn reduce_debt() -> Weight {
        (118_115_000 as Weight)
            .saturating_add(DbWeight::get().reads(2 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn transfer_pallet_admin() -> Weight {
        (64_232_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn accept_pallet_admin() -> Weight {
        (70_817_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
    }
    fn set_feed_creator() -> Weight {
        (69_569_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
    fn remove_feed_creator() -> Weight {
        (67_570_000 as Weight)
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}