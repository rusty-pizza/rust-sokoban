fn main() {
    if let Err(err) = sokoban::run() {
        println!("{}", err)
    }
}
