fn split_at_demo(a: &[i32; 5], i: usize) -> (&[i32], &[i32]) {
    assert!(i <= a.len());
    a.split_at(i)
}

fn main() {
    let a = [1, 2, 3, 4, 5];
    let _ = split_at_demo(&a, 2);
}
