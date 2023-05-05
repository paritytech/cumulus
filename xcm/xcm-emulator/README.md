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

Please refer to [example crate source-code](example/src/lib.rs) for usage examples.
