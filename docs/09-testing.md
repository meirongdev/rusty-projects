# 09 测试与可测试的设计

> 测试写起来不难，难的是**让代码可测**。这一节后半段比前半段重要。
> 对应项目：[`guessing_game`](../guessing_game)

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
- `assert_eq!` 要求参与比较的类型实现 `PartialEq`（能比较）和 `Debug`（失败时能打印实际值），所以被测的枚举都 `#[derive(Debug, PartialEq, Eq)]`（见[笔记 07](./07-traits.md)）。

**binary crate（只有 `main.rs`）一样可以写单元测试**，不需要为了测试强行拆出 `lib.rs`。

常用命令：

```bash
cargo test -p guessing_game        # 只跑某个成员
cargo test --workspace             # 跑全部
cargo test rejects                 # 只跑名字里含 "rejects" 的测试
cargo test -- --nocapture          # 让被测代码的 println! 也显示出来
```

> **单元测试 vs 集成测试**：上面这种和源码写在一起的是单元测试。另一种放在项目根的 `tests/` 目录下，每个文件是独立 crate，只能调用 `pub` 接口——它测的是「使用者视角的公开 API」。本仓库目前只有单元测试。

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
- 输入从注入的 reader 读取——真实运行传 `io::stdin().lock()`，测试传 `BufReader::new(&b"..."[..])`（`impl Trait` 参数见[笔记 07](./07-traits.md)）。

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

## 延伸阅读

- [The Book ch11 — Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [The Book ch11-03 — Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)（单元测试与集成测试的分工）
