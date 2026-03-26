fn main() {
    let a = [1, 2, 3, 4, 5];
    let mut i = 0;
    while i < 5 {
        let _b = a.get(i);
        i = i + 1;
    }

    while i < 7 {
        let _c = a.get(i);
        i = i + 1;
    }
}
