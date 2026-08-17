# guessing_game

经典 Rust 入门项目——猜数字游戏。程序随机生成一个 1~100 的秘密数字，玩家最多猜 7 次，每次提示「太小」「太大」还是「猜中」。它覆盖了从 Hello World 迈向真实小程序所需的几乎全部基础。

## 运行

```bash
cargo run -p guessing_game    # 在仓库根目录
cargo test -p guessing_game   # 跑 11 个测试
```

交互示例（`>` 后面是你输入的内容）：

```
Guess the number! I'm thinking of a number between 1 and 100.
--- Attempt 1/7 ---
> 50
50 is too small!
--- Attempt 2/7 ---
> abc
That's not a number. Enter an integer between 1 and 100. Try again.
--- Attempt 2/7 ---
> 101
101 is outside the 1..=100 range. Try again.
--- Attempt 2/7 ---
> 75
75 is too big!
--- Attempt 3/7 ---
```

留意中间那两次：**输错了不消耗猜测机会**，计数停在 `Attempt 2/7`。这一点看似理所当然，实现起来却是本项目最值得琢磨的坑（见笔记 04）。

## 练到的知识点

| 笔记 | 在这个项目里的体现 |
|------|-------------------|
| [01 Cargo 与工具链](../docs/01-cargo-and-toolchain.md) | 在 `[dependencies]` 里引入 `rand = "0.10"`——第一次接触 crates.io 生态 |
| [02 变量、常量与基本类型](../docs/02-variables-and-types.md) | `const RANGE_START/END/MAX_GUESSES`、`let mut line`、选 `u32` 导致负数被判为「不是数字」 |
| [03 所有权与借用](../docs/03-ownership-and-borrowing.md) | `read_line(&mut line)` 可变借用、`cmp(&secret_number)` 不可变借用、`parse_guess(input: &str)` 只借不拿 |
| [04 控制流](../docs/04-control-flow.md) | **为什么用 `while` 而不是 `for`**：`for` 由迭代器驱动，`continue` 会推进它，非法输入照样消耗机会 |
| [05 枚举与模式匹配](../docs/05-enums-and-pattern-matching.md) | `GuessError` / `GameOutcome` 携带数据、`match guess.cmp(...)` 处理 `Ordering` |
| [06 错误处理](../docs/06-error-handling.md) | 自定义错误枚举、`map_err` 转换错误类型、`?` 提前返回、`expect` 何时可接受 |
| [07 Trait](../docs/07-traits.md) | `use rand::RngExt`（不导入就没有 `random_range`）、`#[derive]`、手写 `impl Display`、`&mut impl BufRead` |
| [08 格式化与宏](../docs/08-formatting-and-macros.md) | 通篇使用内联格式化参数 `println!("{guess} is too small!")` |
| [09 测试与可测试的设计](../docs/09-testing.md) | 把随机性和输入注入成参数，让**整局游戏**都可以被 `assert_eq!` |

## 代码结构

```
parse_guess(&str) -> Result<u32, GuessError>       纯逻辑，不碰 I/O
play(secret_number, &mut impl BufRead) -> GameOutcome   一整局，不确定性全靠注入
main()                                              只负责生成随机数 + 接上真实 stdin
```

这个分层不是为了好看，而是为了让 `play` 成为确定性函数——测试喂固定输入就能复现整局游戏。展开讲在[笔记 09](../docs/09-testing.md#真正的难点让代码可测)。

## 扩展练习

- 把「最多 7 次」改成可配置：从命令行参数读取（先手写 `std::env::args()`，再试试 `clap` crate）。
- 猜中后询问「再来一局吗」，把整局游戏包进外层循环。
- 记录并展示历史猜测（用 `Vec<u32>` 收集，结束时打印出来）。
- 把 `parse_guess` 和 `play` 拆到 `src/lib.rs`，`main.rs` 只留 I/O，感受 lib crate 与 bin crate 的分工以及 `pub` 可见性的作用。
- 给 `GuessError` 实现 `std::error::Error`，再把 `main` 改成 `fn main() -> Result<(), Box<dyn Error>>`，体验用 `?` 贯穿整个程序。
- 进阶：把「秘密数字怎么生成」也注入——`fn play(rng: &mut impl RngExt, ...)`，用固定种子的 RNG 让随机性本身也可复现，还能单独测试「秘密数字一定落在范围内」。
