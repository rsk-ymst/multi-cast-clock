ID_A = 1
ID_B = 2

IP_ADDR_A = 127.0.0.1:8080
IP_ADDR_B = 127.0.0.2:8080

BIN = target/debug/full-order-mult.exe

A: BIN
	cargo run -- $(ID_A) $(ID_B) $(IP_ADDR_A) $(IP_ADDR_B)

B: BIN
	cargo run -- $(ID_B) $(ID_A) $(IP_ADDR_B) $(IP_ADDR_A)

ope: BIN
	cargo test

BIN:
	cargo build

clean:
	cargo clean
