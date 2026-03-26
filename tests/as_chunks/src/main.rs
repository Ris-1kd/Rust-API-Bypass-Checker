fn main() {
    let v = [1,2,3,4,5,6];
    let _a = v.as_chunks(2);

    let index_list = [2,3,4];
    for index in index_list {
        let _b = v.as_chunks(index_list[index]);
    }
}