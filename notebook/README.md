# notebook

一个把 Rust **所有权与借用**（外加一点点生命周期）练到肌肉记忆里的小笔记本。
它不解决什么实际问题——意义全在**读代码时看清每一项数据到底归谁**。

`Entry` 持有两条拥有的 `String`，`Notebook` 是每个 `Entry` 的唯一所有者。
每一条公开方法的签名都是一道所有权题：参数该拿**值**还是**借用**，
返回值该**交还所有权**还是**借出去**，都由所有权规则说了算。

## 运行

```bash
cargo run -p notebook    # 在仓库根目录
cargo test -p notebook   # 跑 29 个测试（单元 + 集成 + doctest）
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
| **移动** | `add(&mut self, title: String, body: String)` 把两个拥有值的所有权拿进函数里，调用后原变量失效 |
| **`&` 借用 + 生命周期** | `get(&self, title: &str) -> Option<&Entry>`：返回的 `&Entry` 只和 `&self` 绑定，不和查找键 `&str` 绑定 |
| **`&mut` 借用** | `get_mut(&mut self, ...)` 是全场唯一能改数据的入口；和 `get` 不能同时持有 |
| **Copy 的引用** | `list(&self) -> Vec<&str>` 平摊出去的 `&str` 是 `Copy` 的，遍历零 clone |
| **把所有权还回来** | `remove(&mut self, ...) -> Option<Entry>` 把 `Entry` 从集合里**交还**给调用方 |
| **拿值还是拿借用，由被调方决定** | `run(input: impl BufRead, out: &mut impl Write)`：`lines(self)` 会消耗 reader 所以按值拿，`Write` 只要 `&mut self` 所以借就够 |

对应笔记：[03 所有权与借用](../docs/03-ownership-and-borrowing.md)、
[10 生命周期（深入所有权）](../docs/10-lifetimes-and-more-ownership.md)。

## 坑

- **`get` 的借用和 `get_mut` 冲不冲突，看的是「后面还用不用」，不是「声明了没」。**
  拿到 `get` 的结果之后**还要再用它**，中间夹一个 `get_mut` 才会报 E0502；
  如果那个借用之后再没被碰过，同一段代码编得过去。这是 NLL，也是本项目最容易想当然
  的一处——两半分别被 `Notebook::get_mut` 文档里的 `compile_fail` doctest 和单元测试
  `borrow_ends_at_last_use` 钉着，展开讲在[笔记 10](../docs/10-lifetimes-and-more-ownership.md#坑)。
- **`add` 收的是 `String` 而不是 `&str`，这是故意的。** 改成 `&str` 也能编（内部 `to_string()`
  一下就行），但那样一来「谁在为这份数据买单」就藏进了实现里；收 `String` 是把
  「所有权归我了」写在签名上。**签名就是合同**——调用方读签名就知道自己的变量还能不能用。
- **标题不去重。** 底下是 `Vec` 不是 `HashMap`，`add` 两条同名笔记会老老实实存两条，
  而 `get` / `remove` 只认最先加进去的那条。想要 map 语义得自己在调用前判断，见下面的扩展练习。

## 代码结构

```
src/lib.rs           Notebook + Entry：纯逻辑，零 I/O，谁的签名就是什么所有权课
src/main.rs          读命令 -> 打印结果，I/O 只在最外层（复习 docs/09）
tests/public_api.rs  集成测试：独立 crate，只看得见 pub 的东西
```

输入和输出都是 `run` 的参数，所以**整段 REPL 交互**都能在测试里跑：喂一个 `&[u8]`，
用 `Vec<u8>` 接住它说过的每句话，再断言。这是[笔记 09](../docs/09-testing.md) 那三层
可测性设计在本项目里的落法。

## 扩展练习

- 底层换成 `HashMap<String, Entry>`：公开签名几乎一个字都不用改，实现里 `iter().find(...)`
  换成 `HashMap::get` / `HashMap::remove`——体会「借来的键去查、返回借来的值」这套思想
  换了容器也一样。顺手把上面那条「标题不去重」的坑一并解决掉。
- 加一个 `has_title(&self, title: &str) -> bool`：只借不动数据，和 `remove` 的
  「交还所有权」正好形成对照。
- 让 `add` 在标题重复时报错（返回 `Result`），体会「先 `get` 判断存在、再 `add` 写入」
  时借用的先后顺序——这正是 `&self` 和 `&mut self` 不能重叠会卡住你的地方。
- 进阶：试着把 `Entry` 的字段从 `String` 改成 `&str`，看编译器把你逼到哪一步——
  `Entry` 会因此带上生命周期参数（`Entry<'a>`），`Notebook` 跟着也要带，最后连
  「笔记本能活多久」都取决于那些字符串是从哪借来的。要摆脱这层传染只有交出所有权
  （用回 `String`）或者引入 `Rc` / `RefCell`。**这就是所有权想逼你避开的复杂度。**
- 挑战：在 `main` 里同时打印 `get` 的结果和 `list` 的结果——两个 `&self` 借用可以共存。
  然后按下面的顺序改一版，观察编译器教你什么：

  ```rust
  let view = nb.get("hello");          // 借用开始
  nb.edit_body("hello", "x".into());   // 要 &mut self
  println!("{view:?}");                // ← 把这一行删掉，上面那行就合法了
  ```

  **删掉最后一行和留着最后一行，是两种完全不同的结果**——这就是上面「坑」第一条说的事。