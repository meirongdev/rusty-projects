# 学习笔记

按主题整理的 Rust 笔记。**一个概念只在这里讲一次**，项目 README 和代码注释都不重复它，只负责链接过来。

## 阅读顺序

| # | 笔记 | 一句话 | 练手项目 |
|---|------|--------|---------|
| 01 | [Cargo 与工具链](./01-cargo-and-toolchain.md) | 工具链、edition/MSRV、crate 与 workspace、依赖版本、常用命令 | `hello_cargo` |
| 02 | [变量、常量与基本类型](./02-variables-and-types.md) | `let`/`mut`/`const`、整型、类型标注、`&str` vs `String` | `guessing_game` |
| 03 | [所有权与借用](./03-ownership-and-borrowing.md) | 三条规则、移动、`&` 与 `&mut` | `guessing_game` |
| 04 | [控制流](./04-control-flow.md) | `for`/`while`/`loop` 的区别、`continue` 陷阱、范围、`return` vs `break` | `guessing_game` |
| 05 | [结构体与方法](./05-structs-and-methods.md) | 字段与 `impl` 块、方法 vs 关联函数、`self` 的三种写法、字段可见性 | `notebook` |
| 06 | [枚举与模式匹配](./06-enums-and-pattern-matching.md) | 带数据的枚举、`match` 表达式与穷尽性 | `guessing_game` |
| 07 | [错误处理](./07-error-handling.md) | `Result`、`expect`、自定义错误类型、`map_err`、`?` | `guessing_game` |
| 08 | [Trait](./08-traits.md) | trait 必须导入、`derive`、`Display` vs `Debug`、`impl Trait` 参数 | `guessing_game` |
| 09 | [格式化与宏](./09-formatting-and-macros.md) | 宏为什么带 `!`、内联格式化参数 | `hello_cargo` |
| 10 | [测试与可测试的设计](./10-testing.md) | `#[cfg(test)]`、断言、把不确定性注入成参数 | `guessing_game` |
| 11 | [生命周期（深入所有权）](./11-lifetimes-and-more-ownership.md) | 借来的引用能活多久、省略规则、`&self` 返回值 | `notebook` |

01 和 09 配合 `hello_cargo` 读，02、04、06–08 配合 `guessing_game` 读，05 和 11 配合 `notebook` 读，
03 和 10 两个项目都用得上——所有权那条线在 `guessing_game` 里只是零散用到，在 `notebook` 里才是主角，
而全仓库的结构体也只有 `notebook` 里那两个。

## 每篇笔记的结构

- **概念**：是什么、怎么写；
- **本仓库里的体现**：直接引用项目里的真实代码，而不是造一段脱离上下文的示例；
- **坑**：写错会怎样，编译器会说什么；
- **延伸阅读**：官方 Book / 标准库文档的对应章节。

## 写新笔记时

- 一个概念只能有一处讲解。如果发现自己在重复某段解释，说明它应该被抽成笔记然后被链接。
- 例子优先从仓库现有代码里取——脱离上下文的玩具代码留不下印象。
- 「踩过的坑」比「语法罗列」值钱，语法查官方文档就够了，坑不会写在那里。
