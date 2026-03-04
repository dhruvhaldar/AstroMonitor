fn main() {
    let s = " \u{200B}=cmd";
    let t = s.trim_start_matches(|c: char| c.is_whitespace() || c.is_control() || c == '\u{200B}');
    println!("{:?}", t);
}
