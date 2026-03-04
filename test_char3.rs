fn main() {
    let s = "\u{FEFF}=cmd";
    let t = s.trim_start();
    println!("{:?}", t);
}
