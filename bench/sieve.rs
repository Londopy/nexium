const LIMIT: usize = 100000000;
fn main() {
    let mut composite = vec![false; LIMIT];
    let mut count = 0usize;
    let mut i = 2;
    while i < LIMIT {
        if !composite[i] {
            count += 1;
            let mut j = i * i;
            while j < LIMIT { composite[j] = true; j += i; }
        }
        i += 1;
    }
    println!("{}", count);
}
