fn main() {
    let args = std::env::args();
    if args.len() > 1 {
        println!("Usage: rslox [script]")
    }
}
