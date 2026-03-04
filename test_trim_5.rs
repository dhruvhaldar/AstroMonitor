fn main() {
    let s = "\u{200B}=cmd";
    let t = s.trim_start();
    println!("{:?}", t);
}
