use std::time::Instant;

fn is_malicious_csv_payload_old(s: &str) -> bool {
    for c in s.chars() {
        if c.is_whitespace()
            || c.is_control()
            || c == '\u{FEFF}'
            || ('\u{200B}'..='\u{200F}').contains(&c)
            || ('\u{202A}'..='\u{202E}').contains(&c)
            || ('\u{2066}'..='\u{2069}').contains(&c)
        {
            continue;
        }
        return c == '=' || c == '+' || c == '-' || c == '@';
    }
    false
}

fn is_malicious_csv_payload_new(s: &str) -> bool {
    if let Some(&b) = s.as_bytes().first() {
        if b.is_ascii_alphanumeric() {
            return false;
        }
    }
    for c in s.chars() {
        if c.is_whitespace()
            || c.is_control()
            || c == '\u{FEFF}'
            || ('\u{200B}'..='\u{200F}').contains(&c)
            || ('\u{202A}'..='\u{202E}').contains(&c)
            || ('\u{2066}'..='\u{2069}').contains(&c)
        {
            continue;
        }
        return c == '=' || c == '+' || c == '-' || c == '@';
    }
    false
}

fn main() {
    let s = "Sirius - The brightest star";
    let mut old_time = 0;
    let mut new_time = 0;

    let start = Instant::now();
    for _ in 0..1_000_000 {
        std::hint::black_box(is_malicious_csv_payload_old(std::hint::black_box(s)));
    }
    old_time = start.elapsed().as_micros();

    let start = Instant::now();
    for _ in 0..1_000_000 {
        std::hint::black_box(is_malicious_csv_payload_new(std::hint::black_box(s)));
    }
    new_time = start.elapsed().as_micros();

    println!("Old: {}us, New: {}us", old_time, new_time);
}
