BIN := "rlox-bytecode"

repl:
    cargo run --bin {{BIN}}

file file:
    cargo run --bin {{BIN}} -- {{file}}

miri file *flags:
    MIRIFLAGS="-Zmiri-disable-isolation {{flags}}" cargo +nightly miri run --bin {{BIN}} -- {{file}}