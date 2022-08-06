BIN := "rlox-bytecode"

repl:
    cargo run --bin {{BIN}}

miri file:
    MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri run --bin {{BIN}} -- {{file}}