# 02 变量、常量与基本类型

> 默认不可变是 Rust 的第一个「反直觉」设定，它是所有权规则能成立的前提。
> 对应项目：[`guessing_game`](../guessing_game)

## `let` 默认不可变

```rust
let x = 5;
x = 6;          // ❌ 编译错误：cannot assign twice to immutable variable

let mut line = String::new();
input.read_line(&mut line)?;   // ✅ line 声明成 mut 才能被写入
```

不可变是**默认**，可变是**显式选择**。这不是为了限制你，而是让「这个值会变吗」变成读代码时一眼可见的信息——后面的借用规则（[笔记 03](./03-ownership-and-borrowing.md)）正是建立在这个区分上的。

> **遮蔽（shadowing）**：用 `let` 重复声明同名变量会创建一个新变量，而不是修改旧的，且可以换类型：
> ```rust
> let guess = "42";
> let guess: u32 = guess.trim().parse().unwrap();   // 同名，但是新变量、新类型
> ```
> 经典版猜数字游戏用它把 `String` 转成 `u32`。本仓库改成了返回 `Result` 的 `parse_guess` 函数，所以没有用到遮蔽——但读别人的代码时会经常遇到。

## `const`：编译期常量

```rust
const RANGE_START: u32 = 1;
const RANGE_END: u32 = 100;
const MAX_GUESSES: u32 = 7;
```

和 `let` 的区别：

- **必须**显式标注类型（`let` 可以推断，`const` 不行）；
- 值必须在编译期就能算出来，不能是函数调用的结果；
- 不能加 `mut`——它压根不是变量；
- 惯例用 `SCREAMING_SNAKE_CASE` 命名，可以声明在任意作用域，包括模块顶层。

把 `1` / `100` / `7` 这些魔法数字提成常量后，范围检查、提示文案、随机数生成引用的是同一个来源，改规则时只改一处。

## 整型与类型标注

Rust 的整型按「有无符号 + 位宽」组合：`i8`/`i16`/`i32`/`i64`/`i128`/`isize` 与 `u8`/`u16`/`u32`/`u64`/`u128`/`usize`。默认整型是 `i32`。

本仓库选了 `u32`（猜测值不可能为负），这个选择有一个**刻意的副作用**：

```rust
let number: u32 = input.trim().parse().map_err(|_| GuessError::NotANumber)?;
```

`"-5"` 解析成 `u32` 会失败，所以负数被归进了「不是数字」而不是「超出范围」。这是取舍，不是 bug——测试里专门钉住了这个行为。

注意这行的 `: u32` 不只是注释，它**驱动了类型推断**：`parse()` 能解析成任何实现了 `FromStr` 的类型，编译器正是从这个标注（以及函数返回类型 `Result<u32, GuessError>`）反推出该解析成 `u32`。去掉标注会直接报 `type annotations needed`。

## `&str` 与 `String`

这是初学者最常撞上的一对类型：

| | `&str` | `String` |
|---|--------|----------|
| 本质 | **借用**的字符串切片 | **拥有**数据的字符串 |
| 内存 | 指向别处（字面量在编译期就固定在程序里） | 堆上分配 |
| 可变 | 长度不可变 | 可增长 |
| 典型来源 | `"Hello, world!"`、`&some_string` | `String::new()`、`.to_string()` |

本仓库两者都出现了：`hello_cargo` 里的 `"Hello, world!"` 是 `&str`；`guessing_game` 用 `String::new()` 接收输入，而 `parse_guess(input: &str)` 只借用不拥有——**函数参数优先用 `&str`**，这样传 `String` 和字面量都能直接调用。

这对区别的本质是所有权，继续看[笔记 03](./03-ownership-and-borrowing.md)。

## 延伸阅读

- [The Book ch03-01 — Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)
- [The Book ch03-02 — Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
