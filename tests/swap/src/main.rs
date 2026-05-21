fn swap_demo(v: &mut [i32; 6], i: usize) {
    let j = i + 1;
    assert!(i < v.len());
    assert!(j < v.len());
    v.swap(i, j);
}

fn main() {
    let mut v = [1, 2, 3, 4, 5, 6];
    swap_demo(&mut v, 2);
}
