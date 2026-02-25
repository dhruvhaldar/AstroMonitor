fn main() {
    let mut count = 0;
    for c in '\0'..='\u{10FFFF}' {
        if c.is_control() {
            if !((c >= '\u{0000}' && c <= '\u{001F}') || (c >= '\u{007F}' && c <= '\u{009F}')) {
                println!("Found unexpected control char: U+{:04X}", c as u32);
                count += 1;
            }
        }
    }
    if count == 0 {
        println!("Assumption correct: Cc contains only C0 and C1 controls.");
    }
}
