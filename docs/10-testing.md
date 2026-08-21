# 10 测试与可测试的设计

> 测试写起来不难，难的是**让代码可测**。这一节后半段比前半段重要。
> 对应项目：[`guessing_game`](../guessing_game)、[`notebook`](../notebook)

## 单元测试的标准骨架

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_range() {
        assert_eq!(parse_guess("0"), Err(GuessError::OutOfRange(0)));
    }
}
```

- `#[cfg(test)]` 表示这个模块**只在 `cargo test` 时编译**，`cargo build` 出来的可执行文件里不含测试代码。
- `use super::*;` 把父模块（这里就是 `main.rs` 本身）的内容引进来，测试才能调用 `parse_guess`。这也是为什么单元测试能访问**私有**函数——它和被测代码在同一个 crate 里。
- `#[test]` 标记一个测试函数；`assert_eq!` / `assert_ne!` / `assert!` 是最常用的三个断言。
- `assert_eq!` 要求参与比较的类型实现 `PartialEq`（能比较）和 `Debug`（失败时能打印实际值），所以被测的枚举都 `#[derive(Debug, PartialEq, Eq)]`（见[笔记 08](./08-traits.md)）。

**binary crate（只有 `main.rs`）一样可以写单元测试**，不需要为了测试强行拆出 `lib.rs`。

常用命令：

```bash
cargo test -p guessing_game        # 只跑某个成员
cargo test --workspace             # 跑全部
cargo test rejects                 # 只跑名字里含 "rejects" 的测试
cargo test -- --nocapture          # 让被测代码的 println! 也显示出来
cargo test --doc                   # 只跑文档测试
cargo test --lib                   # 只跑单元测试（不含集成测试和 doctest）
```

> 上面这种和源码写在一起的是**单元测试**。Rust 还有另外两种，见下面「三种测试」一节。

## 三种测试，各测一件事

Rust 自带三种测试，`cargo test` 一条命令全跑。它们不是重复劳动，各自回答不同的问题：

| 种类 | 放在哪 | 能看到什么 | 回答什么问题 |
|------|--------|-----------|-------------|
| 单元测试 | 源文件里的 `#[cfg(test)] mod tests` | 同 crate 的**私有**项 | 内部实现对不对 |
| 集成测试 | 项目根的 `tests/*.rs` | 只有 `pub` 的东西 | 公开 API 够不够用 |
| 文档测试（doctest） | `///` 文档里的代码块 | 只有 `pub` 的东西 | 文档里的例子有没有过期 |

### 集成测试：换成使用者的眼睛

`tests/` 下**每个文件都是独立的 crate**，得像外人一样 `use` 你的库：

```rust
// notebook/tests/public_api.rs
use notebook::{Entry, Notebook};
```

正因为看不见私有项，它能发现单元测试永远发现不了的一类问题：**公开接口漏了东西**。
`Notebook` 的 `entries` 字段是私有的，如果 `remove` 忘了标 `pub`，单元测试照过，
集成测试直接编译不过。

> 集成测试只对 **library crate** 有意义——`tests/` 里的文件要 `use` 你的 crate，
> 而 binary crate 没法被 `use`。这就是「逻辑放 lib、`main.rs` 只留薄薄一层 I/O」
> 除了可测之外的第二个理由。

### doctest：让文档里的例子自己保证自己没过期

`///` 文档里用三个反引号围起来的 Rust 代码块，**`cargo test` 会真的去编译并运行它**：

```rust
/// # Examples
///
/// ```
/// use notebook::Notebook;
///
/// let mut nb = Notebook::new();
/// nb.add("todo".to_string(), "write a test".to_string());
/// assert!(nb.remove("todo").is_some());
/// ```
```

改了 API 却忘了改文档，测试立刻变红。**文档从此不会撒谎**——这是 Rust 相比大多数语言
独有的一件事，值得养成习惯。

### `compile_fail`：把报错断言交给编译器

教所有权时最常写的一句话是「这样写编译不过」。麻烦在于，这句话本身没人验证——
本仓库就栽过：`notebook` 里一条注释写着「取消注释这行会编译失败」，实际上取消之后
照样编译通过（原因见[笔记 11](./11-lifetimes-and-more-ownership.md#坑)）。

代码块加个 `compile_fail` 就能把这句话交给编译器来管——它要求这段**必须编译失败**，
一旦哪天真能编过了，`cargo test` 就红给你看：

````rust
/// ```compile_fail
/// let mut nb = Notebook::new();
/// let title = String::from("hello");
/// nb.add(title, "body".to_string()); // title 的所有权进了 add
/// println!("{title}");               // ERROR: borrow of moved value
/// ```
````

**坑：`compile_fail` 只保证「编不过」，不保证「因为你想的那个原因编不过」。** 例子里
打错一个字母，它照样「通过」。可以写成 ` ```compile_fail,E0382 ` 把期望的错误码标出来，
但要清楚**这个错误码在 stable 上不会被校验**（写成别的码也照过），它只是给读者看的注释。
所以 `compile_fail` 的例子要写得尽量短，短到一眼能看出它为什么编不过。

## 真正的难点：让代码可测

不确定性是测试的敌人。猜数字游戏有两个不确定性来源：**随机的秘密数字**和**用户输入**。如果它们都埋在 `main()` 里，这个游戏就什么都测不了。

本仓库分两层把它们赶了出去。

### 第一层：把纯逻辑摘出来

```rust
fn parse_guess(input: &str) -> Result<u32, GuessError>
```

它不碰 stdin、不打印任何东西，同样的输入必然得到同样的输出。**把 I/O 挡在函数外面**，是让代码可测试最常用的一招。

### 第二层：把不确定性变成参数

```rust
fn play(secret_number: u32, input: &mut impl BufRead) -> GameOutcome
```

- 秘密数字由调用方传入——随机性挪到了 `main` 里；
- 输入从注入的 reader 读取——真实运行传 `io::stdin().lock()`，测试传 `BufReader::new(&b"..."[..])`（`impl Trait` 参数见[笔记 08](./08-traits.md)）。

两个不确定性都变成参数之后，`play` 就是确定性函数：喂什么输入、配什么秘密数字，结果唯一。**这就是依赖注入（dependency injection）最朴素的样子**——不需要任何框架，一个函数参数就够了。

`main` 于是只剩两件事：

```rust
fn main() {
    let secret_number = rand::rng().random_range(RANGE_START..=RANGE_END);
    play(secret_number, &mut io::stdin().lock());
}
```

### 第三层：让结局可断言

```rust
enum GameOutcome {
    Won { attempts: u32 },
    OutOfGuesses,
    InputClosed,
}
```

`play` 返回枚举而不是只打印文案，测试断言的就是**结果**，不用去比对输出字符串（那种测试一改文案就碎）。

> **有时候「结局」就是那段输出本身。** `notebook` 的 REPL 没有 `GameOutcome` 那样的结果类型——
> 它干的事就是打印。这时把**输出也变成参数**即可：
>
> ```rust
> fn run(input: impl BufRead, out: &mut impl Write) -> io::Result<()>
> ```
>
> 真实运行传 `io::stdout().lock()`，测试传一个 `Vec<u8>` 收着，跑完再断言里面有什么。
> `&[u8]` 实现了 `BufRead`、`Vec<u8>` 实现了 `Write`，两头都不需要临时文件或测试框架。
> 注意 `out` 是 `&mut` 借用而 `input` 是按值拿走——因为 `BufRead::lines(self)` 会消耗
> reader，而 `Write` 的方法只要 `&mut self`。**参数拿值还是拿借用，由被调用方最少需要
> 什么决定**，不是风格问题。

## 收获：坑都被钉住了

[笔记 04](./04-control-flow.md) 讲的那些坑，现在每一个都有测试盯着：

```rust
#[test]
fn invalid_input_keeps_attempt() {
    // abc 不是数字、101 越界，都不算一次猜测，第 1 次有效输入就猜中。
    let mut input = BufReader::new(&b"abc\n101\n42\n"[..]);
    assert_eq!(play(42, &mut input), GameOutcome::Won { attempts: 1 });
}
```

本项目一共 11 个测试：6 个覆盖 `parse_guess` 与 `Display`，5 个覆盖**整局游戏**——输错不消耗机会、EOF 不死循环、用光 7 次、第几次猜中。

`notebook` 则把上面三种测试都用上了：单元测试盯内部行为，`tests/public_api.rs` 盯公开 API，
`///` 里的 doctest 盯文档，其中三个 `compile_fail` 例子专门钉住「这样写会报错」。

## 延伸阅读

- [The Book ch11 — Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [The Book ch11-03 — Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)（单元测试与集成测试的分工）
- [The Book ch14-02 — Documentation Comments as Tests](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#documentation-comments-as-tests)
- [rustdoc book — Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)（`compile_fail` / `should_panic` / `ignore` 等属性的完整列表）
