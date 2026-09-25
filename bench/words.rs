use std::collections::HashMap;
const WORDS: i64 = 10000000;
fn main() {
    let mut counts: HashMap<String, i64> = HashMap::new();
    let mut seed: u64 = 42;
    let mut total: i64 = 0;
    for _ in 0..WORDS {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let k = (seed >> 33) % 50000;
        let word = format!("w{}", k);
        *counts.entry(word).or_insert(0) += 1;
        total += 1;
    }
    println!("{} {}", counts.len(), total);
}
