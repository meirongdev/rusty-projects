# rusty-projects

一个循序渐进的 Rust 学习仓库。每个子目录是一个可独立运行的小项目，[`docs/`](./docs) 是按主题整理的学习笔记。

## 这个仓库怎么用

代码只是练习场，**[`docs/`](./docs) 才是主线**。三层内容各司其职，互不重复：

| 位置 | 回答什么 |
|------|---------|
| [`docs/`](./docs) | 「这个 Rust 概念是什么、怎么用、有什么坑」——一个概念只有一处讲解 |
| 项目 `README.md` | 「这个例子怎么跑、练到了哪些概念」 |
| 代码注释 | 「这一行为什么要这么写」——只记本地决策，不复述概念 |

## 项目一览

按推荐顺序阅读，后一个项目默认你已经消化了前一个。

| # | 项目 | 主题 | 对应笔记 |
|---|------|------|---------|
| 1 | [`hello_cargo`](./hello_cargo) | Cargo 项目结构与构建流程 | [01](./docs/01-cargo-and-toolchain.md)、[08](./docs/08-formatting-and-macros.md) |
| 2 | [`guessing_game`](./guessing_game) | 从 Hello World 迈向真实小程序 | [01](./docs/01-cargo-and-toolchain.md)–[09](./docs/09-testing.md) |

## 前置条件

需要 **Rust 1.85 或更高版本**（edition 2024 与 `rand` 0.10 的下限）：

```bash
rustc --version   # 低于 1.85 请先 rustup update
```

根目录的 `rust-toolchain.toml` 会让 rustup 自动切到 stable 并装好 `rustfmt` / `clippy`，不用手动配置。为什么 CI 要单独用 1.85 跑一遍，见[笔记 01](./docs/01-cargo-and-toolchain.md#edition-与-msrv)。

## 怎么跑

本仓库是一个 **Cargo workspace**，在根目录即可操作全部成员：

```bash
cargo run -p guessing_game   # 运行指定成员
cargo test --workspace       # 跑所有成员的测试
cargo fmt --all              # 格式化整个 workspace
```

也可以进到子目录，像独立项目一样操作（Cargo 会自动向上找到 workspace 根）：

```bash
cd guessing_game && cargo run
```

## 提交前的自检

CI（[`.github/workflows/ci.yml`](./.github/workflows/ci.yml)）跑的就是下面三条，本地过了就不会在 CI 上翻车：

```bash
cargo fmt --all --check
cargo lint            # 本仓库定义的别名，= cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`cargo lint` 不是内置命令，而是 [`.cargo/config.toml`](./.cargo/config.toml) 里的别名，理由见[笔记 01](./docs/01-cargo-and-toolchain.md#用别名给长命令起短名字)。

## 新增一个例子

在**仓库根目录**执行：

```bash
cargo new my_example
```

Cargo 会自动把 `my_example` 追加进根 `Cargo.toml` 的 `members`，并让新成员的 `version` / `edition` / `license` / `rust-version` 全部继承 workspace（也不会另生成一份 `Cargo.lock`）。剩下两件事需要手动做：

1. 删掉 `cargo new` 在新目录里生成的嵌套 `.git/` 和 `.gitignore`——本仓库已经有自己的了：
   ```bash
   rm -rf my_example/.git my_example/.gitignore
   ```
2. 照着现有项目写一份 `README.md`：**运行方式 → 练到哪些笔记 → 扩展练习**。遇到笔记里还没有的新概念，**写进 `docs/` 再链接过来**，不要写在项目 README 里。

最后跑一遍上面的自检三件套，确认新成员是干净的。

## 仓库约定

- **每个例子都要能通过 `fmt` + `clippy` + `test`。** 学习者会把仓库代码当范本，范本自己得是干净的。
- **`Cargo.lock` 提交进版本库**，公共字段在根 `Cargo.toml` 的 `[workspace.package]` 里声明一次——理由见[笔记 01](./docs/01-cargo-and-toolchain.md)。

## License

[MIT](./LICENSE)
