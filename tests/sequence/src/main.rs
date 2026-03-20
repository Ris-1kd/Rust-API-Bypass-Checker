//  用于测试在非循环的时序敏感的条件下, 一个原始变量派生出多个局部变量不能共享抽象值的情况下, 其抽象值该怎么追溯的问题.
//  有多种情况:
//  1. 第一种不包含循环, 最终的分析结果是一个具体的值, 这没什么好说的;
//  2. 第二种两个连续的循环, 局部local能有两个范围, 但是可能与循环条件分支本身相关;
//  3. 第三种情况, 我新建一个变量传递一下, 将条件分支的变量和传参的变量分开, 看看local会怎么追踪;

fn main() {

    let a = [1,2,3];
    let b = [1,2,3,4,5];
    let mut i = 0;
    let mut index = 0;
    let mut mul = 1;
    while i < 3 {
        index = i;
        let _a = a.get(index);
        i = i + 1;
        mul = mul * 2;
    }
    
    while i < 6 {  
        index = i;
        let _b = b.get(index);
        i = i + 1;
        mul = mul * 2;
    }

    println!("The final mul is: {}", mul);
}