# notebook

一个把 Rust **所有权与借用**（外加一点点生命周期）练到肌肉记忆里的小笔记本。
它不解决什么实际问题——意义全在**读代码时看清每一项数据到底归谁**。

`Entry` 持有两条拥有的 `String`，`Notebook` 是每个 `Entry` 的唯一所有者。
每一条公开方法的签名都是一道所有权题：参数该拿**值**还是**借用**，
返回值该**交还所有权**还是**借出去**，都由所有权规则说了算。

## 运行

```bash
cargo run -p notebook    # 在仓库根目录
cargo test -p notebook   # 跑 11 个测试
```

交互示例（`>` 后面是你输入的内容）：

```
Commands:
  add <title> <body...>   add a note
  get <title>             show one note
  edit <title> <body...>  replace the body
  delete <title>          remove a note
  list                    list all titles
  help / ?                this help
  quit / exit             leave

> add hello World, hello!
added: hello
> add rust Ownership is the core
added: rust
> list
  - hello
  - rust
> get hello
hello
  World, hello!
> edit rust Borrowing too
edited: rust
> delete hello
deleted: hello (World, hello!)
> quit
bye!
```

## 练到的知识点

这个项目是**所有权规则本身的教学现场**：每看一个签名，就是在复习对照文档。

| 概念 | 在本项目里的体现（一个方法一个） |
|------|----------------------------------|
| **移动** | `add(&mut self, title: String, body: String)` 把两个拥有值的所有权拿进函数据，调用后原变量失效 |
| **`&` 借用 + 生命周期** | `get(&self, title: &str) -> Option<&Entry>`：返回的 `&Entry` 只和 `&self` 绑定，不和查找键 `&str` 绑定 |
| **`&mut` 借用** | `get_mut(&mut self, ...)` 是全场唯一能改数据的入口；和 `get` 不能同时持有 |
| **Copy 的引用** | `list(&self) -> Vec<&str>` 平摊出去的 `&str` 是 `Copy` 的，遍历零 clone |
| **把所有权还回来** | `remove(&mut self, ...) -> Option<Entry>` 把 `Entry` 从集合里**交还**给调用方 |

对应笔记：[03 所有权与借用](../docs/03-ownership-and-borrowing.md)、
[10 生命周期（深入所有权）](../docs/10-lifetimes-and-more-ownership.md)。

## 坑

- 想 `get` 拿到一个借用后，立刻 `edit` 同一条笔记——编译期直接拒绝（borrow conflict）。
  单独一个 `&self` 或 `&mut self` 都行，同一个对象二者不能共存。
- 把 `takes_ownership` 传值误写成传 `&str` 就能糊弄过去一眼看不出问题——
  但记住：**藏数据的人（`Notebook`）决定谁拥有数据**，签名就是合同。

## 代码结构

```
src/lib.rs   Notebook + Entry：纯逻辑，零 I/O，谁的签名就是什么所有权课
src/main.rs  读命令 -> 打印结果，I/O 只在最外层（复习 docs/09）
```

## 扩展练习

- 标题改成 `HashMap<String, String>` 存储，看看 `get` / `remove` 的签名几乎不用改，
  但实现换成 `HashMap::lookup`——体会「借用来查找」的思想到处一样。
- 给 `remove` 加一个返回 `bool` 的 `has_title` 判断，做成查找不移动数据。
- 让标题去重：重复 `add` 同标题时报错（先 `get` 判断存在性再 `add`，体会借用的先后顺序）。
- 进阶延伸：把 `Entry` 改成 `&str` 借用不行（生命周期会变短），非要自己做就得用
  生命周期标注或 `Rc`/`RefCell`——这正是所有权想逼你避开的复杂度。
- 挑战：`main` 里一次同时打印 `get` 的结果和 `list` 的结果——两个 `&self` 借用可以共存；
  再试打印 `get` 的结果后立刻 `edit`，观察编译器的报错教你什么。