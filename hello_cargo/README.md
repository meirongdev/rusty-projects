# hello_cargo

最简单的 Rust 程序：打印一行 `Hello, world!`。它存在的意义不是这行输出，而是把 Cargo 的项目结构和构建流程完整走一遍。

## 运行

```bash
cargo run -p hello_cargo   # 在仓库根目录
cargo run                  # 或者进到本目录再跑
```

预期输出：

```
Hello, world!
```

## 练到的知识点

| 笔记 | 在这个项目里的体现 |
|------|-------------------|
| [01 Cargo 与工具链](../docs/01-cargo-and-toolchain.md) | `Cargo.toml` 清单与 workspace 字段继承、`src/main.rs` 作为 binary crate 入口、`cargo check` / `build` / `run` 的分工 |
| [08 格式化与宏](../docs/08-formatting-and-macros.md) | `println!` 的 `!` 为什么代表宏、`{}` 占位符 |

> 本目录下只有 `Cargo.toml` 和 `src/`，没有 `Cargo.lock` 和 `target/`——因为本仓库是一个 workspace，这两样在根目录被所有成员共享。独立项目里它们会出现在项目自己的目录下。

## 扩展练习

- `cargo build` 之后直接运行 `../target/debug/hello_cargo`，确认它就是 `cargo run` 跑的东西。
- 对比 `cargo build` 与 `cargo build --release` 的产物大小和所在目录。
- 故意把 `println!` 写成 `println`，读一遍编译器的报错和建议——Rust 的报错信息本身就是很好的老师。
- 用 `let` 定义一个变量，再用内联格式化参数 `println!("{name}")` 把它打印出来。
