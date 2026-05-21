fn get_demo(a: &[i32; 5], i: usize) -> Option<&i32> {
    assert!(i < a.len());
    a.get(i)
}

fn main() {
    let a = [1, 2, 3, 4, 5];
    let _ = get_demo(&a, 2);
}
