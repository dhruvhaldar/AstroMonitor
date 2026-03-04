fn main() {
    let s = " \u{200B}=cmd";
    println!("{:?}", s.trim_start());
}
