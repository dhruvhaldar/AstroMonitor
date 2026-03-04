fn main() {
    let s = "\x08=cmd"; // backspace
    println!("{:?}", s.trim_start());
}
