# Rust 异步编程

本书旨在成为一本详尽的 Rust 异步编程指南，涵盖从入门到进阶的内容。

本书已经很久没有维护，也缺乏关注。我们目前正在努力将其更新并大幅改进！由于我们正在进行一些重大更改，内容可能会有些混乱，部分内容可能会重复或缺失，等等。请多多包涵，很快就会好起来的 :-) 要了解我们的计划并告诉我们你的想法，请参见 [issue 224](https://github.com/rust-lang/async-book/issues/224)。

## 环境要求

本异步书使用 [`mdbook`](https://rust-lang.github.io/mdBook/index.html) 构建，你可以使用 cargo 来安装它。

```
cargo install mdbook
cargo install mdbook-linkcheck2
```

[`mdbook`]: https://github.com/rust-lang/mdBook

## 构建
要生成最终的书本，请运行 `mdbook build`，它会在 `book/` 目录下生成。

```
mdbook build
```

## 开发

在编写时，能够实时查看你的更改会非常方便，`mdbook serve` 会启动一个本地 Web 服务器来提供本书的预览。

```
mdbook serve
```

## 贡献

本仓库是 [`async-book`](https://github.com/rust-lang/async-book) 的中文翻译版，如发现翻译有误或内容落后于上游仓库，欢迎参与贡献。
