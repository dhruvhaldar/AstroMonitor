use std::time::Instant;
use std::hint::black_box;

fn check_control_naive(s: &str) -> bool {
    s.chars().any(|c| c.is_control())
}

fn check_control_mixed(s: &str) -> bool {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b < 32 || b == 127 {
            return true;
        }
        if b >= 128 {
            return s[i..].chars().any(|c| c.is_control());
        }
    }
    false
}

fn check_control_optimized(s: &str) -> bool {
    let bytes = s.as_bytes();
    let len = bytes.len();
    for i in 0..len {
        let b = bytes[i];
        if b < 32 || b == 127 {
            return true;
        }
        // C1 control characters (U+0080..U+009F) are encoded as 0xC2 0x80..0x9F
        if b == 0xC2 && i + 1 < len {
            let next = bytes[i+1];
            if next >= 0x80 && next <= 0x9F {
                return true;
            }
        }
    }
    false
}

fn main() {
    let s_ascii = "Sirius - The brightest star in the night sky";
    let s_mixed = "Sirius ★ - The brightest star"; // Includes multi-byte char
    let s_long_mixed = "Sirius ★".repeat(100);

    let iterations = 1_000_000;

    // 1. Naive (chars().any())
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(check_control_naive(black_box(s_mixed)));
    }
    println!("Naive (Mixed) took {:?}", start.elapsed());

    // 2. Current Mixed Approach (as implemented in codebase)
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(check_control_mixed(black_box(s_mixed)));
    }
    println!("Current Mixed (Mixed) took {:?}", start.elapsed());

    // 3. Optimized Byte Scan
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(check_control_optimized(black_box(s_mixed)));
    }
    println!("Optimized (Mixed) took {:?}", start.elapsed());


    // Long string test
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(check_control_mixed(black_box(&s_long_mixed)));
    }
    println!("Current Mixed (Long) took {:?}", start.elapsed());

    let start = Instant::now();
    for _ in 0..iterations {
        black_box(check_control_optimized(black_box(&s_long_mixed)));
    }
    println!("Optimized (Long) took {:?}", start.elapsed());
}
