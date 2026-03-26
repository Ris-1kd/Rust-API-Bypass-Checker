fn main (){
    let mut v = vec![1,2,3,4,5,6];
    let mut i = 0;

    while i + 1 < v.len() {
        v.swap(i, i+1);
        i = i + 1;
    }
}