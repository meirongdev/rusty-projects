# 09 格式化与宏

> `println!` 后面那个 `!` 不是装饰——它表示这是一个**宏**，在编译期展开成代码。
> 对应项目：[`hello_cargo`](../hello_cargo)、[`guessing_game`](../guessing_game)

## 宏和函数的区别

```rust
println!("Hello, world!");
```

`!` 表示 `println!` 是宏而非普通函数。宏在编译期展开，所以能做到函数做不到的事：

- **接受可变数量、可变类型的参数**——普通 Rust 函数做不到；
- **在编译期检查格式字符串**：占位符数量和参数对不上会直接编译失败，而不是等到运行时。

一组常用的格式化宏，语法完全一致：

| 宏 | 输出到 |
|----|--------|
| `println!` / `print!` | 标准输出（带 / 不带换行） |
| `eprintln!` / `eprint!` | 标准错误 |
| `format!` | 返回一个 `String` |
| `write!` / `writeln!` | 写进任意 `Formatter` 或 writer（实现 `Display` 时用的就是它） |
| `panic!` | 崩溃并打印消息 |

## 占位符：位置参数与内联捕获

老写法是按位置传参：

```rust
println!("{} + {} = {}", 1, 2, 3);
```

从 **Rust 1.58**（2022-01）起，`{}` 里可以直接写**变量名**，编译器会从作用域里捕获：

```rust
let name = "world";
println!("Hello, {name}!");                    // 等价于 println!("Hello, {}!", name)
println!("{guess} is too small!");
println!("{number} is outside the {RANGE_START}..={RANGE_END} range.");
```

**限制：只对变量名有效。** 表达式仍然要用位置参数：

```rust
println!("{}", guess + 1);       // ✅
println!("{guess + 1}");         // ❌ 编译不过
```

这个语法对上面所有格式化宏都适用。本仓库通篇用的都是内联写法。

> **这是「版本」而不是「edition」带来的能力**，两者的区别见[笔记 01](./01-cargo-and-toolchain.md#edition-与-msrv)。
> 内联捕获跟 `edition = "2021"` 没关系——把一个 `edition = "2015"` 的 crate 用 1.58+ 的编译器编，
> `println!("Hello, {name}!")` 照样能跑。会混淆是因为它俩年份挨得近：
> **edition 决定语言方言，编译器版本决定「有没有这个功能」。**

## `{}` 与 `{:?}`

- `{}` 走 `Display`，给最终用户看；
- `{:?}` 走 `Debug`，给程序员看；`{:#?}` 是换行缩进的美化版。

两者是不同的 trait，详见[笔记 08](./08-traits.md)。

## 延伸阅读

- [The Book ch01-02 — Hello, World!](https://doc.rust-lang.org/book/ch01-02-hello-world.html)
- [`std::fmt` 模块文档](https://doc.rust-lang.org/std/fmt/)（格式化语法的完整规格：宽度、精度、对齐、进制）
