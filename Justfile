BIN := "rlox-bytecode"

repl:
    cargo run --bin {{BIN}}

file file:
    cargo run --bin {{BIN}} -- {{file}}

miri file:
    MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri run --bin {{BIN}} -- {{file}}