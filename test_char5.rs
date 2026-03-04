fn main() {
    let s = "\x08=cmd";
    let t = s.trim_start_matches(|c: char| c.is_whitespace() || c.is_control() || c == '\u{200B}' || c == '\u{FEFF}');
    println!("{:?}", t);
}
