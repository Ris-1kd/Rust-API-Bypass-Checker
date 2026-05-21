fn checked_add_demo(a: i32, b: i32) -> i32 {
    assert!(a >= 0);
    assert!(a <= 100);
    assert!(b >= 0);
    assert!(b <= 100);
    let result = a.checked_add(b).unwrap();
    result
}

fn main() {
    let result = checked_add_demo(40, 2);
    println!("result: {:?}", result);
}
