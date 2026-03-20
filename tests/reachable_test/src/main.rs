// main.rs
#![allow(dead_code)]


#[derive(Clone, Debug, Default)]
pub struct Packet {
    id: u32,
    payload: Vec<u8>,
}

/// 自定义 trait：用于测试“trait 方法（AssocFn）”的枚举与过滤
pub trait Analyzer {
    fn normalize(&mut self);
    fn score(&self) -> usize;
}

impl Packet {
    /// 结构体本身的方法 1：构造
    pub fn new(id: u32, payload: Vec<u8>) -> Self {
        Self { id, payload }
    }

    /// 结构体本身的方法 2：使用 Vec::swap
    pub fn swap_ends(&mut self) {
        if self.payload.len() >= 2 {
            let last = self.payload.len() - 1;
            self.payload.swap(0, last);
        }
    }

    /// 结构体本身的方法 3：使用 slice::split_at（安全）
    pub fn split_payload(&self, mid: usize) -> (&[u8], &[u8]) {
        let m = mid.min(self.payload.len());
        self.payload.split_at(m)
    }

    /// 结构体本身的方法 4：给 payload 做一个轻微扰动（也包含 split_at_mut + swap）
    pub fn perturb(&mut self) {
        self.swap_ends();

        if self.payload.len() >= 4 {
            // split_at_mut：可变切分
            let (left, right) = self.payload.split_at_mut(2);
            left.swap(0, 1);
            right.reverse();
        }
    }
}

impl Analyzer for Packet {
    /// trait 方法 1：调用结构体方法 + split_at_mut + swap
    fn normalize(&mut self) {
        self.perturb();

        // 再做一次“局部 swap”，让调用点更明显
        if self.payload.len() >= 2 {
            self.payload.swap(0, 1);
        }
    }

    /// trait 方法 2：调用 split_at（安全）并结合自定义函数 checksum
    fn score(&self) -> usize {
        let (a, b) = self.split_payload(2);
        checksum(a) ^ (checksum(b) << 1)
    }
}

/// 自定义自由函数 1：checksum（纯数值逻辑）
pub fn checksum(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .fold(0usize, |acc, &x| acc.wrapping_add(x as usize))
}

/// 自定义自由函数 2：处理逻辑，调用 trait 方法
pub fn process(mut p: Packet) -> (Packet, usize) {
    p.normalize();
    let s = p.score();
    (p, s)
}

/// 自定义自由函数 3：演示调用 std::mem::swap + 派生的 clone/debug
pub fn demo() {
    // std::mem::swap（与 Vec::swap 不同的 swap 调用点）
    let mut a: u32 = 1;
    let mut b: u32 = 2;
    std::mem::swap(&mut a, &mut b);

    let p = Packet::new(7, vec![10, 20, 30, 40, 50]);

    // 这里会触发派生的 Clone::clone
    let (p2, s) = process(p.clone());

    // 这里会触发派生的 Debug::fmt
    println!("a={a}, b={b}, p2={p2:?}, score={s}");

    let mut v = vec![1,2,3,4,5];
    v.swap(0,1);
}


pub fn foo()  {
    let a = [1,2,3,4,5];
    let _b = a.get(0);
}


pub fn foo1(){
    foo();
}


fn main() {
    demo();
    foo1();
}
