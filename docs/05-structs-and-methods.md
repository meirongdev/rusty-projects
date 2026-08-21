# 05 结构体与方法

> 枚举是「几种可能里挑一种」，结构体是「几样东西凑成一个」。行为则不写在结构体里，
> 而是挂进 `impl` 块——而 `self` 的三种写法，就是[笔记 03](./03-ownership-and-borrowing.md)
> 那套所有权规则换了个位置重写一遍。
> 对应项目：[`notebook`](../notebook)（它排在第 3 个，但本篇只需要读它的签名，不用先跑起来）

## 结构体：把几个值捆成一个类型

最常用的是具名字段结构体，本仓库两个都是这种：

```rust
pub struct Entry {
    pub title: String,
    pub body: String,
}
```

- 每个字段都要写类型，**没有推断**（和 `const` 一样，见[笔记 02](./02-variables-and-types.md)）；
- 构造时必须把字段给全，写成 `字段名: 值`；
- 字段名和变量名同名时可以省掉一半，这叫字段初始化简写（field init shorthand）：

```rust
// notebook 的 add 里就是这么写的
self.entries.push(Entry { title, body }); // 等价于 Entry { title: title, body: body }
```

另外两种形态本仓库没用到，但读别人的代码会遇到：

| 形态 | 写法 | 怎么访问字段 | 典型用途 |
|------|------|-------------|---------|
| 元组结构体 | `struct Meters(f64);` | `m.0` | newtype：给裸类型套一层，避免把米和秒搞混 |
| 单元结构体 | `struct Marker;` | 没有字段 | 不占空间，只用来挂 trait |

**struct 还是 enum？** 一个值同时具备所有字段就用 struct（AND），一个值只能是若干形态之一
就用 enum（OR，见[笔记 06](./06-enums-and-pattern-matching.md)）。`Entry` 同时有标题和正文，
所以是 struct；`GuessError` 要么是「不是数字」要么是「越界」，所以是 enum。

## `impl` 块：方法与关联函数

数据和行为是**分开写**的：`struct` 只声明字段，行为全部放进 `impl`。

```rust
impl Notebook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}
```

两者的区别只有一条——**第一个参数是不是 `self`**：

| | 第一个参数 | 怎么调用 | 例子 |
|---|-----------|---------|------|
| **方法**（method） | `self` 的某种形式 | `.` | `nb.len()` |
| **关联函数**（associated function） | 不是 `self` | `::` | `Notebook::new()` |

`Self`（大写 S）是「当前 `impl` 的那个类型」的别名：`-> Self` 就是 `-> Notebook`，
`Self::default()` 就是 `Notebook::default()`。用 `Self` 的好处是类型改名时这些地方不用跟着改。

`new` **不是关键字，也不属于任何 trait**，纯粹是社区惯例——「返回一个新实例的关联函数就叫
`new`」。编译器不认识这个名字，你完全可以叫 `create`，只是没人这么写。

> `impl Notebook { ... }` 这种叫**固有实现**（inherent impl）：方法直接长在类型上。
> `impl fmt::Display for GuessError { ... }` 那种是「为类型实现某个 trait」，是另一回事，
> 见[笔记 08](./08-traits.md)。

## `self` 的三种写法，就是参数的三种拿法

这是本篇和[笔记 03](./03-ownership-and-borrowing.md) 的接缝，也是最值得记住的一条：
**`&self` 只是 `self: &Self` 的简写。** 所以选哪一个，用的还是笔记 03 那条规则——
看被调用方最少需要什么。

| 写法 | 全称 | 意思 | 调用之后对象还在吗 |
|------|------|------|------------------|
| `&self` | `self: &Self` | 不可变借用，只能读 | 在 |
| `&mut self` | `self: &mut Self` | 可变借用，能改字段 | 在 |
| `self` | `self: Self` | 拿走所有权 | **没了** |

`Notebook` 的方法正好凑成一张对照表：

```rust
fn len(&self) -> usize                                          // 只读
fn is_empty(&self) -> bool                                      // 只读
fn get(&self, title: &str) -> Option<&Entry>                    // 只读，还借一个引用出去
fn list(&self) -> Vec<&str>                                     // 只读
fn add(&mut self, title: String, body: String)                  // 要往 Vec 里塞
fn get_mut(&mut self, title: &str) -> Option<&mut Entry>        // 要借出可改的引用
fn edit_body(&mut self, title: &str, new_body: String) -> bool  // 要改
fn remove(&mut self, title: &str) -> Option<Entry>              // 要从 Vec 里拿走一项
```

规律一眼可见：**改不改数据，决定 `&self` 还是 `&mut self`**。返回值那一栏归生命周期管，
是[笔记 11](./11-lifetimes-and-more-ownership.md) 的题目——`get` 在源码里其实手写成
`get<'a>(&'a self, ...) -> Option<&'a Entry>`，那是留给笔记 11 做省略规则对照的，
按省略规则它和上面这行完全等价。

这里没有一个方法拿 `self`——因为没有哪个操作会让整个笔记本「用完就废」。想看拿 `self`
的真实例子得去标准库：`notebook` 的 `run` 用的 `BufRead::lines(self)` 就是，它会**消耗**掉
reader，所以 `run` 只能按值接收 reader 而不是借用（那段取舍写在 `run` 的文档注释里，
配套讲解见[笔记 10](./10-testing.md)）。

## 本仓库里的体现：字段可见性

```rust
pub struct Entry {
    pub title: String,   // pub
    pub body: String,    // pub
}

pub struct Notebook {
    entries: Vec<Entry>, // 没有 pub
}
```

**`pub struct` 只是说「这个类型外面能用」，字段默认仍然是私有的**，要一个个标 `pub`。
仓库里这两个结构体刻意选了相反的做法：

- `Entry` 字段全 `pub`——它就是一袋数据，使用者拿到 `&mut Entry` 就该能直接改，
  集成测试 `get_mut_lets_a_caller_edit_in_place` 正是这么用的；
- `Notebook.entries` 私有——增删改查只能走那几个方法，外面碰不到 `Vec` 本身。于是
  「标题不去重」这类规则才有地方可守，将来把容器换成 `HashMap` 也不会惊动任何调用方。

这个私有字段还撑起了[笔记 10](./10-testing.md) 那条关于集成测试的说法：`tests/public_api.rs`
是独立 crate，`entries` 在它眼里根本不存在——**「只看得见 `pub` 的东西」是靠字段私有兑现的，
不是靠君子协定**。

## 坑

下面每条的报错都是实际编译出来的，不是凭印象写的。

**1. `pub struct` 不等于字段 `pub`，外面根本构造不出来。**

```rust
let nb = Notebook { entries: Vec::new() };
// ❌ error[E0451]: field `entries` of struct `Notebook` is private
```

这正是私有字段的类型**必须**提供构造入口的原因——没有 `Notebook::new()`，外面就造不出
一个 `Notebook` 来。

**2. 方法漏写一个 `&`，对象会被吃掉。**

```rust
impl Notebook {
    fn len(self) -> usize {  // 本该是 &self
        self.entries.len()
    }
}

let _ = nb.len();
let _ = nb.len();
// ❌ error[E0382]: use of moved value: `nb`
//    note: `Notebook::len` takes ownership of the receiver `self`, which moves `nb`
```

报错里那句 note 就是本篇的中心思想：**`self` 会移动接收者**。这个错第一次遇到会很困惑——
明明只是想「读一下长度」，怎么把对象弄没了。少打一个 `&` 而已。

**3. `&self` 的方法里改不了字段。**

```rust
fn bump(&self) { self.count += 1; }
// ❌ error[E0594]: cannot assign to `self.count`, which is behind a `&` reference
//    help: consider changing this to be a mutable reference
```

编译器把改法直接写在 help 里了：换成 `&mut self`。

**4. 同一个 `impl` 里调用兄弟方法，必须写 `self.`。**

```rust
fn is_empty(&self) -> bool { len() == 0 }
// ❌ error[E0425]: cannot find function `len` in this scope
//    help: consider using the method on `Self`
```

`impl` 块不是一个作用域，方法不会自动进到当前命名空间。得写 `self.len() == 0`。

**5. 有 `len()` 就得有 `is_empty()`，clippy 盯着这一条。**

只写 `len` 不写 `is_empty`，`cargo lint` 会直接失败（本仓库把警告升级成了错误）：

```
warning: struct `Notebook` has a public `len` method, but no `is_empty` method
  = note: `#[warn(clippy::len_without_is_empty)]` on by default
```

`Notebook` 两个都有，就是被这条 lint 要求出来的。理由也站得住：调用方写 `nb.is_empty()`
比 `nb.len() == 0` 清楚。

**6. 只写 `new()` 不给 `Default`，clippy 同样会说话。**

```
warning: you should consider adding a `Default` implementation for `Notebook`
  = note: `#[warn(clippy::new_without_default)]` on by default
```

本仓库的做法是**反过来**的——先 `#[derive(Default)]`，再让 `new()` 委托给它：

```rust
#[derive(Debug, Default)]
pub struct Notebook {
    entries: Vec<Entry>,
}

impl Notebook {
    pub fn new() -> Self {
        Self::default()
    }
}
```

「空的笔记本是什么样」于是只有一处定义，将来给 `entries` 换容器也只改一个地方。
单元测试 `new_and_default_agree` 钉住了两者一致。

## 动手体会

照[笔记 03](./03-ownership-and-borrowing.md) 的路子，挑一个改掉再读报错：

- 把 `Notebook::get` 的 `&self` 改成 `self`，看调用方第二次用 `nb` 时编译器说什么；
- 把 `edit_body` 的 `&mut self` 改成 `&self`，看它指着哪一行；
- 给 `entries` 加上 `pub`，然后在 `tests/public_api.rs` 里直接读 `nb.entries.len()`——
  它**能**编过，然后再想想为什么仓库偏偏不这么做。

## 延伸阅读

- [The Book ch05 — Using Structs to Structure Related Data](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [The Book ch05-03 — Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
- [The Rust Reference — Implementations](https://doc.rust-lang.org/reference/items/implementations.html)
- [The Rust Reference — Visibility and Privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
