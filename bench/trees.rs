const MIN_DEPTH: i64 = 4;
const MAX_DEPTH: i64 = 17;
struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}
fn make(depth: i64) -> Box<Node> {
    if depth == 0 {
        Box::new(Node { left: None, right: None })
    } else {
        Box::new(Node { left: Some(make(depth - 1)), right: Some(make(depth - 1)) })
    }
}
fn count(n: &Node) -> i64 {
    match (&n.left, &n.right) {
        (Some(l), Some(r)) => 1 + count(l) + count(r),
        _ => 1,
    }
}
fn main() {
    let mut trees: i64 = 0;
    let mut nodes: i64 = 0;
    {
        let stretch = make(MAX_DEPTH + 1);
        nodes += count(&stretch);
        trees += 1;
    }
    let long_lived = make(MAX_DEPTH);
    let mut depth = MIN_DEPTH;
    while depth <= MAX_DEPTH {
        let iterations = 1i64 << (MAX_DEPTH - depth + MIN_DEPTH);
        for _ in 0..iterations {
            let t = make(depth);
            nodes += count(&t);
            trees += 1;
        }
        depth += 2;
    }
    nodes += count(&long_lived);
    trees += 1;
    println!("{} {}", trees, nodes);
}
