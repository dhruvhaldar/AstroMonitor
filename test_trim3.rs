fn main() {
    let s = "\x09\x0b\x0c\x0d@cmd";
    let t = s.trim_start();
    println!("{:?}", t);
}
