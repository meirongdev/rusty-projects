# 01 Cargo 与工具链

> Rust 的工程层：谁来编译、编译哪些东西、依赖从哪来。
> 对应项目：[`hello_cargo`](../hello_cargo)、[`guessing_game`](../guessing_game)

## 工具链的三个角色

| 名字 | 职责 |
|------|------|
| `rustup` | 工具链管理器，负责装/切换 Rust 版本 |
| `rustc` | 编译器，真正把源码变成机器码 |
| `cargo` | 构建系统 + 包管理器，日常只跟它打交道 |

平时几乎不直接调用 `rustc`——`cargo build` 会替你组织好所有参数再调它。

`rustup` 会在项目里查找 `rust-toolchain.toml`，找到就自动切到指定工具链：

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

有了这个文件，clone 下来就不用手动 `rustup override` 或 `rustup component add`。本仓库写 `stable` 而不是钉死版本号——教程仓库应该跟着最新 stable 走，真正的版本下限交给下面的 MSRV 声明。

## edition 与 MSRV

这是两个容易混的概念：

- **edition**（`edition = "2024"`）：*语言方言*。每三年一版，允许引入会破坏兼容的语法调整。不同 edition 的 crate 可以互相依赖，所以升级 edition 不会撕裂生态。
- **rust-version**（`rust-version = "1.85"`）：*最低支持的 Rust 版本（MSRV）*。工具链比它旧时，Cargo 会直接给出一句清晰的报错，而不是抛出一堆看不懂的语法错误。

本仓库要求 1.85，因为 edition 2024 需要 1.85+，`rand` 0.10 的下限也正好是 1.85。

**MSRV 声明必须靠 CI 验证。** 你本地装的是最新 stable，编译通过只能证明「最新版能编」，永远证明不了「1.85 也能编」。所以 [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) 专门开了一个 job 用 1.85 跑 `cargo check`——防止新代码悄悄用上比 1.85 更新的 API 而没人发现。

## package、crate 与 workspace

- **crate**：编译的最小单位。分两种——
  - **binary crate**：入口 `src/main.rs`，编译出可执行文件；
  - **library crate**：入口 `src/lib.rs`，不能直接运行，是给别人 `use` 的。
- **package**：一份 `Cargo.toml` 管辖的范围，可以包含最多一个 lib crate 和多个 bin crate。
- **workspace**：把多个 package 组织在一起，共享同一份 `target/` 和 `Cargo.lock`。

本仓库根目录是一份**虚拟清单（virtual manifest）**：它自己不是 package，没有 `[package]` 段，只负责组织成员。

```toml
[workspace]
members = ["hello_cargo", "guessing_game"]

# 虚拟清单不会从成员的 edition 推断 resolver，必须显式写出来，
# 否则 Cargo 会回落到老的 resolver = "1" 并给出警告。
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
rust-version = "1.85"
```

workspace 带来三件事：

1. 所有成员共享根目录下唯一一份 `target/` 和 `Cargo.lock`（**所以成员目录里看不到它们，这是正常的**）；
2. 在根目录可以一次性操作全部成员（`cargo test --workspace`）；
3. 公共字段写一次，成员用 `xxx.workspace = true` 继承。

成员的清单因此可以短到只剩一个名字：

```toml
[package]
name = "guessing_game"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
rand = "0.10"
```

独立项目（不在 workspace 里）则直接写字面量：`version = "0.1.0"`。

## 依赖与版本号

```toml
rand = "0.10"
```

- `"0.10"` 等价于 `"^0.10"`，表示「>=0.10.0 且 <0.11.0」的**范围**。Cargo 在这个范围里挑一个具体版本。
- 挑中的精确版本记录在 `Cargo.lock` 里。想知道锁到了哪个版本，看这个文件，或跑 `cargo tree`。
- **`Cargo.lock` 要提交进版本库。** 本仓库以 binary crate 为主，官方建议提交它，保证任何人 clone 下来都构建出完全一致的依赖版本。CI 里的 `--locked` 参数会校验 lock 与所有 `Cargo.toml` 一致，防止改了清单却忘了更新 lock。

> **版本升级会改 API——这本身就是一课。**
> 老教程（包括较早印次的《The Rust Programming Language》）写的是 `rand = "0.8"`，配 `rand::thread_rng()` 和 `.gen_range(...)`。从 0.9 起这两个名字改成了 `rand::rng()` 和 `.random_range(...)`；到 0.10，提供 `random_range` 的 trait 又从 `Rng` 挪到了 `RngExt`（细节见[笔记 07](./07-traits.md)）。
> 照着老教程写会直接编译不过。遇到这种情况，去 [docs.rs](https://docs.rs/rand) 查**当前版本**的文档、或读 crate 的 CHANGELOG，是比搜索引擎更可靠的习惯。

## 常用命令

| 命令 | 作用 |
|------|------|
| `cargo new <name>` | 创建新项目 |
| `cargo check` | 只做类型检查、不生成产物，**速度最快**，写代码时的主力命令 |
| `cargo build` | 编译生成可执行文件（debug 模式，产物在 `target/debug/`） |
| `cargo build --release` | 开启优化编译，慢得多但产物快得多 |
| `cargo run` | 编译并运行 |
| `cargo test` | 编译并跑测试 |
| `cargo fmt` | 按官方风格格式化代码 |
| `cargo clippy` | 静态检查，给出比编译器更进一步的改进建议 |
| `cargo tree` | 打印依赖树 |

在 workspace 根目录时，加 `-p <name>` 指定单个成员，或加 `--workspace` 作用于全部成员。

## 用别名给长命令起短名字

参数太长记不住时，可以在 `.cargo/config.toml` 里起**别名**——这是 Cargo 自带的能力，不用装任何东西，在仓库任意子目录下都生效：

```toml
[alias]
lint = "clippy --workspace --all-targets -- -D warnings"
```

之后敲 `cargo lint` 即可。这两个参数值得起别名，是因为都容易漏又都很重要：

- `--all-targets` 让 clippy 连 `#[cfg(test)]` 里的测试代码一起检查；
- `-- -D warnings` 把所有警告升级为错误，避免警告越积越多。

**别名只能包装一条 cargo 子命令**，不能用 `&&` 串联——写 `ci = "fmt --all --check && cargo clippy"` 会直接报 `unexpected argument '&&'`。想把多条压成一条得引入 `just` 之类的任务运行器，本仓库刻意不引入：学 Rust 的阶段，直接敲 cargo 原生命令比记住一层包装更有价值，这些命令在任何 Rust 项目里都通用。

同理也不提供 Makefile——Cargo 本身就是构建系统，再包一层 `make build: cargo build` 只是多一层要维护的空壳。

## 延伸阅读

- [The Book ch01 — Getting Started](https://doc.rust-lang.org/book/ch01-00-getting-started.html)
- [The Book ch07 — Packages, Crates and Modules](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
- [The Book ch14 — More About Cargo](https://doc.rust-lang.org/book/ch14-00-more-about-cargo.html) / [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Cargo Reference — Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) / [Configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- [Edition Guide — Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
