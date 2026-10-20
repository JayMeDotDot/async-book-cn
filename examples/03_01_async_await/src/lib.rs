#![allow(unused)]
#![cfg(test)]

mod async_fn_and_block_examples {
use std::future::Future;
// ANCHOR: async_fn_and_block_examples

// `foo()` 返回一个实现了 `Future<Output = u8>` 的类型。
// `foo().await` 将产生一个 `u8` 类型的值。
async fn foo() -> u8 { 5 }

fn bar() -> impl Future<Output = u8> {
    // 这个 `async` 块产生的类型实现了
    // `Future<Output = u8>`。
    async {
        let x: u8 = foo().await;
        x + 5
    }
}
// ANCHOR_END: async_fn_and_block_examples
}

mod async_lifetimes_examples {
use std::future::Future;
// ANCHOR: lifetimes_expanded
// 这个函数:
async fn foo(x: &u8) -> u8 { *x }

// 等同于这个函数:
fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { *x }
}
// ANCHOR_END: lifetimes_expanded

async fn borrow_x(x: &u8) -> u8 { *x }

#[cfg(feature = "never_compiled")]
// ANCHOR: static_future_with_borrow
fn bad() -> impl Future<Output = u8> {
    let x = 5;
    borrow_x(&x) // ERROR：`x` 的存活时间不够长
}

fn good() -> impl Future<Output = u8> {
    async {
        let x = 5;
        borrow_x(&x).await
    }
}
// ANCHOR_END: static_future_with_borrow
}

mod async_move_examples {
use std::future::Future;
// ANCHOR: async_move_examples
/// `async` block:
///
/// 多个不同的 `async` 块可以访问同一个局部变量，
/// 只要它们在该变量的作用域内执行
async fn blocks() {
    let my_string = "foo".to_string();

    let future_one = async {
        // ...
        println!("{my_string}");
    };

    let future_two = async {
        // ...
        println!("{my_string}");
    };

    // 运行两个 future 至完成，打印两次 "foo"：
    let ((), ()) = futures::join!(future_one, future_two);
}

/// `async move` block:
///
/// 只有一个 `async move` 块能够访问同一个被捕获的变量，因为
/// 捕获会被移动进由 `async move` 块生成的 `Future` 中。
/// 不过，这也使得该 `Future` 可以活得比变量的原始作用域更久：
fn move_block() -> impl Future<Output = ()> {
    let my_string = "foo".to_string();
    async move {
        // ...
        println!("{my_string}");
    }
}
// ANCHOR_END: async_move_examples
}
