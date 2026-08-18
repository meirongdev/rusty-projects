# notebook — 学习 Rust 所有权的项目（设计）

> 日期：2026-08-18 ｜ 状态：已实现

## 目标

在现有 Cargo workspace 学习仓库 `rusty-projects` 里新增第三个练习项目 `notebook`：
一个把 **所有权 / 借用 / 生命周期** 练到肌肉记忆里的小笔记本。它不是解决实际问题的
程序，而是「谁持有数据始终清楚」的教学现场——每一条公开方法的签名就是一道所有权题。

## 仓库既有约定（必须遵守）

- 每个子目录是可独立运行、**干干净净过 `fmt` + `clippy(-D warnings)` + `test`** 的成员；
- 一个概念只在 `docs/` 里讲一次，项目 README 只负责链接、不复述概念；
- 新概念不在文档里 → 先写进 `docs/` 再链接；
- 项目 README 结构：**运行方式 → 练到哪些笔记 → 扩展练习**；
- `cargo new` 生成后删掉嵌套 `.git/` 与 `.gitignore`，成员登记进根 `Cargo.toml`。

## 结构（Approach 1：lib + bin 单 crate）

```
notebook/
├── Cargo.toml          # workspace 字段继承
├── README.md           # 运行 → 练到哪些笔记 → 扩展练习
└── src/
    ├── lib.rs          # Notebook / Entry 逻辑：纯逻辑，零 I/O，可测
    └── main.rs         # 薄 REPL：读命令 -> 调 lib -> 打印
```

## 领域模型与所有权教学映射

```rust
pub struct Entry { pub title: String, pub body: String } // Entry 持有两条拥有的 String
pub struct Notebook { entries: Vec<Entry> }               // Notebook 是每个 Entry 的唯一所有者
```

| 方法 | 签名 | 演示的所有权行为 |
|------|------|-----------------|
| `new` | `Notebook::new()` | — |
| `add` | `(&mut self, String, String)` | **移动**：值进函数，原变量失效 |
| `get` | `(&self, &str) -> Option<&Entry>` | **借用 + 生命周期**：返回借用挂 `&self` 不挂键 |
| `get_mut` | `(&mut self, &str) -> Option<&mut Entry>` | **`&mut`**：唯一可改入口，与 `get` 互斥 |
| `edit_body` | `(&mut self, &str, String) -> bool` | **移动 + `&mut`**：新正文移入覆盖 |
| `list` | `(&self) -> Vec<&str>` | **Copy 的引用**：`&str` 平摊，零 clone |
| `remove` | `(&mut self, &str) -> Option<Entry>` | **把所有权还回来**：`Entry` 交还调用方 |

`get` 用显式 `'a`（`pub fn get<'a>(&'a self, title: &str) -> Option<&'a Entry>`）并
`#[allow(clippy::needless_lifetimes)]`，作为省略规则的教学对照。

## CLI（main.rs）

REPL：`add <title> <body...>` / `get <title>` / `edit <title> <body...>` / `delete <title>` /
`list` / `help` / `quit`。标题取第一个词，正文取剩余所有词。I/O 全部留在 main，lib 零 I/O
（复习 docs/09 可测性）。`run(input: impl BufRead)` 让整段交互可用注入的输入测试；`parse_command`
为纯函数并有单测。

## 测试（11 个）

lib（8）：add→get 往返；get 缺键为 None；list 按序返回借用标题；edit 原地改；remove 交还
所有权并可改写；remove 缺键 None；生命周期教学测试（返回借用活得比查找键久）；借用互斥对照。
main（3）：解析 add 带正文；短别名（ls/q）；未知/空命令拒绝。

## 文档

- 新增 `docs/10-lifetimes-and-more-ownership.md`：生命周期为什么存在、省略规则、`get` 的
  `&self` 返回教学、坑（不能返回局部引用、借用互斥），含不通过代码 + 编译器报错（放在文档，
  遵循「文档里讲解概念」的约定）。
- `docs/03` 保持不变（三规则/移动/借用）。
- `docs/README.md` 增列第 10 篇；根 `README.md` 项目一览表增 `notebook` 为第 3 项。

## 验证

```bash
cargo fmt --all --check
cargo lint        # = clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
---

## 实现后的调整（2026-08-18 复审）

上面是当初的设计，原样保留。复审时发现并改掉了几处，这里记下差异，免得后来人照着
过期的设计读代码：

| 当初写的 | 实际改成 | 为什么 |
|---------|---------|-------|
| `get` 加 `#[allow(clippy::needless_lifetimes)]` | 删掉这个 `allow` | 实测 clippy 对「靠 `&self` 规则省略」的情形根本不 lint，这个 `allow` 什么都没压住 |
| 用注释说明「取消注释这行会 borrow conflict」 | 改成 `compile_fail,E0502` doctest | 原来那行**取消注释后照样编译通过**（NLL：借用之后没再被用过）。断言交给编译器，才不会再写错 |
| 测试 11 个 | 29 个（10 单元 + 11 单元 + 3 集成 + 5 doctest） | 补了整段 REPL 的测试、`tests/public_api.rs`、以及 doctest |
| `run(input: impl BufRead)` | `run(input: impl BufRead, out: &mut impl Write) -> io::Result<()>` | 原来的 `run` 直接 `println!`，返回 `()`，**根本没法断言**——注释却写着「让整段交互可测」。把输出也注入进去才算数 |
| `parse_command -> Option<Command>` | `-> Result<Command, ParseError>` | 缺参数的 `add` 会被报成「unknown command」，把人引去检查拼写。区分「不认识这条命令」和「参数不够」 |

设计文档本身也搬了个位置：`docs/superpowers/specs/` → `docs/specs/`，路径里不再带工具名。
