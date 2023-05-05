# xcm-emulator

XCM-Emulator is a tool to emulate XCM program execution using
pre-configured runtimes, including those used to run on live
networks, such as Kusama, Polkadot, Statemine et cetera.
This allows for testing cross-chain message passing, verifying
outcomes, weights and side-effects. It is faster than spinning up
a zombienet and as all the chains are in one process debugging using Clion is easy.

## Limitations

As the channels are mocked, using xcm-emulator tests to test
channel setup would not be appropriate.

## Alternatives

If you just wish to test execution of various XCM instructions
against the XCM VM then the `xcm-simulator` (in the polkadot
repo) is the perfect tool for this.

## How to use

1. Setup a relay chain:

```rust
decl_test_relay_chain! {
	pub struct KusamaRelay {
		Runtime = kusama_runtime::Runtime,
		XcmConfig = kusama_runtime::xcm_config::XcmConfig,
		new_ext = kusama_ext(),
	}
}
```

2. Define some parachains:
```rust
decl_test_parachain! {
	pub struct ParachainA {
		Runtime = test_runtime::Runtime,
		RuntimeOrigin = test_runtime::RuntimeOrigin,
		XcmpMessageHandler = test_runtime::XcmpQueue,
		DmpMessageHandler = test_runtime::DmpQueue,
		new_ext = parachain_ext(1),
	}
}
```

3. Setup a network with all the declared parachains and a relay chain:
```rust
decl_test_network! {
	pub struct Network {
		relay_chain = KusamaRelay,
		parachains = vec![
			(1, ParachainA),
		],
	}
}
```

This will wire the in-memory chains together using a mocked network.

4. The relay chain and each parachain needs its `TestExternalities` set up.

5. Finally write some tests:

In the tests it's a good idea to upgrade XCM as the first thing in a test.
Here is a test that upgrades the XCM to v3 and then sends a remark:

```rust

	#[test]
	fn a_test() {
		Network::reset();

		KusamaRelay::execute_with(|| {
			assert_ok!(kusama_runtime::XcmPallet::force_default_xcm_version(
				kusama_runtime::RuntimeOrigin::root(),
				Some(3)
			));
			
    		assert_ok!(test_runtime::PolkadotXcm::send_xcm(
				Here,
				MultiLocation::new(1, X1(Parachain(2))),
				Xcm(vec![Transact {
					origin_kind: OriginKind::SovereignAccount,
					require_weight_at_most: Weight::from_parts(9_000_000, 0),
					call: test_runtime::RuntimeCall::System(
                        frame_system::Call::<test_runtime::Runtime>::remark_with_event {
                            remark: "Hello from Relay!".as_bytes().to_vec(),
                        },
                    ).encode().into(),
				}]),
			));
		});

		ParachainA::execute_with(|| {
			assert_ok!(test_runtime::PolkadotXcm::force_default_xcm_version(
				test_runtime::RuntimeOrigin::root(),
				Some(3)
			));

			assert!(System::events().iter().any(|r| matches!(
				r.event,
				RuntimeEvent::System(frame_system::Event::Remarked { .. })
			)));
		});
	}
```

Please refer to [example crate source-code](example/src/lib.rs) for usage examples.
