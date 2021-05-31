// // This file includes the tests for the chainlink oracle related functions
// use crate::mock::{*};
//
// use frame_support::{assert_ok};
// use crate::mock::ExtBuilder;
//
// #[test]
// fn chainlink_oracle() {
//     ExtBuilder::default()
//         .build()
//         .execute_with(|| {
//             assert_ok!(
//                 ChainlinkOracle::price(0)
//             );
//         });
// }