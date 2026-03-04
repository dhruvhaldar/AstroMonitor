fn main() {
    let s = "\x09\x0b\x0c\x0d=cmd";
    let t = s.trim_start_matches(|c: char| c.is_whitespace() || c.is_control() || c == '\u{200B}');
    println!("{:?}", t);
}
