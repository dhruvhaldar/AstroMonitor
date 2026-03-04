fn main() {
    let s = "A\u{2028}B\u{2029}C";
    let esc = s.escape_debug().to_string();
    println!("{:?}", esc.as_bytes());
}
