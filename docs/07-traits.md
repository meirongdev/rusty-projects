# 07 Trait

> Trait 定义「一类类型共有的行为」。Rust 里方法常常不写在类型里，而是由 trait 提供——**这带来一条新手必踩的规则：trait 不导入，方法就不存在。**
> 对应项目：[`guessing_game`](../guessing_game)

## Trait 必须先导入

```rust
use rand::RngExt;

let secret_number = rand::rng().random_range(RANGE_START..=RANGE_END);
```

删掉那行 `use`，`random_range` 就会「消失」——因为这个方法不属于 `rand::rng()` 返回的类型本身，而是由 `RngExt` trait 提供的。好在编译器会明确告诉你缺了哪个 trait。

注意这里是 `RngExt` 而**不是** `Rng`：`rand` 0.10 把 `random_range` 从 `Rng` 挪到了 `RngExt`，照着老教程写会编译不过（版本变迁的完整脉络见[笔记 01](./01-cargo-and-toolchain.md#依赖与版本号)）。

## `#[derive]`：让编译器帮你实现

```rust
#[derive(Debug, PartialEq, Eq)]
enum GuessError { /* ... */ }
```

常用的几个：

| trait | 作用 |
|-------|------|
| `Debug` | 能被 `{:?}` 打印；`assert_eq!` 失败时靠它显示实际值 |
| `PartialEq` / `Eq` | 能用 `==` 比较；`assert_eq!` 的前提 |
| `Clone` / `Copy` | 能复制 |
| `Default` | 有一个默认值 |

**`assert_eq!` 同时需要 `PartialEq`（能比较）和 `Debug`（失败时能打印）**——这就是本仓库两个枚举都 derive 了它们的原因。

## `Display` 与 `Debug`：给人看 vs 给程序员看

这是两个**不同**的 trait，别混淆：

| | 占位符 | 给谁看 | 怎么来 |
|---|-------|--------|--------|
| `Debug` | `{:?}` | 程序员，调试用 | 通常 `#[derive(Debug)]` |
| `Display` | `{}` | 最终用户 | 只能手写 `impl` |

`Display` 不能 derive，因为「怎么说给人听」没有唯一正确答案：

```rust
impl fmt::Display for GuessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuessError::NotANumber => write!(
                f,
                "That's not a number. Enter an integer between {RANGE_START} and {RANGE_END}."
            ),
            GuessError::OutOfRange(number) => {
                write!(f, "{number} is outside the {RANGE_START}..={RANGE_END} range.")
            }
        }
    }
}
```

实现之后有两个收获：`println!("{error}")` 能直接打印，`.to_string()` 也自动可用（标准库为所有 `Display` 类型统一提供了它）。这是 Rust 里很典型的分工——**类型自己负责怎么展示自己**。

## `impl Trait` 做参数

```rust
fn play(secret_number: u32, input: &mut impl BufRead) -> GameOutcome
```

`&mut impl BufRead` 的意思是「接受任何实现了 `BufRead` 的类型」。于是同一个函数：

- 真实运行时接 `io::stdin().lock()`；
- 测试时接 `BufReader::new(&b"42\n"[..])`，直接喂固定字节。

这是**静态分发**：编译器为每个实际用到的类型各生成一份代码，没有运行时开销。（对应的动态分发写法是 `&mut dyn BufRead`，用一次虚表调用换取「一份代码处理所有类型」。）

> 顺带一个 `std::io` 的小坑：`Stdin` 只实现了 `Read`，带缓冲的 `BufRead` 在 `lock()` 返回的 `StdinLock` 上——所以要先 `.lock()` 再传给 `play`。

## 延伸阅读

- [The Book ch10 — Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [The Book ch10-02 — Traits: Defining Shared Behavior](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) / [`std::io::BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html)
