# Glutton Update
Script for updating Glutton Parachains making use of [`subxt`](https://github.com/paritytech/subxt).
## How to use
- ```cd ./scripts/glutton/update```
- ```cargo build --release```
- Run the binary
    ```bash
	# Example
	./target/release/glutton-update -f 1300 -t 1370 -s 10000000 -c 20000000 -p 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a
	```
- For more info about how to use
	```bash
	./target/release/glutton-update --help
	```
