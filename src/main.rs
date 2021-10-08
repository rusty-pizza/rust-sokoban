fn main() {
    match sokoban::run() {
        Ok(()) => (),
        Err(err) => println!("{}", err),
    }
}
