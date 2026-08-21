# 11 生命周期（深入所有权）

> 上篇 [03 所有权与借用](./03-ownership-and-borrowing.md) 讲清了三条规则、移动、`&`/`&mut`。
> 这一篇回答剩下的那个问题：**借来的引用能活多久？** 答案由编译器用「生命周期」在
> 编译期静态算出，全程不需要垃圾回收。对应项目：[`notebook`](../notebook)

## 为什么引用需要生命周期

借用的安全底线只有一条：**引用不能比它指向的数据活得更久**。一旦数据被 `drop`，指向它的
引用就成了悬垂指针（dangling pointer）。生命周期（lifetime）就是编译器给每个引用标的
「从哪到哪有效」的区间，用它来在编译期证明「绝不会悬垂」——不用等运行时，也不靠程序员自觉。

大多数时候你不用写生命周期——编译器能靠规则自己推导（这就是「省略规则」）。但当函数要
返回一个引用时，编译器必须知道「这个返回值到底挂在哪个输入引用上」，这时规则推不出来，
才需要你出手标注。

## 省略规则（elision）

返回值引用的生命周期按这几条规则确定，通常都不用手写：

- 只有一个输入引用 → 返回值沿用它的生命周期；
- 多个输入引用，但其中一个是 `&self`/`&mut self` → 返回值沿用 `self` 的生命周期；
- 否则 → 编译器推不出来，需要你显式标注。

仓库里的 `notebook::Notebook::get` 正好命中第二条：

```rust
pub fn get<'a>(&'a self, title: &str) -> Option<&'a Entry>
```

这里有两个输入引用（`&self` 和 `&str`），所以返回值只能挂在 `self` 上。代码里那个显式的
`'a` 是**故意写出来**方便对照省略规则的——省略规则本来就能推出同样的签名，写成
`fn get(&self, title: &str) -> Option<&Entry>` 与它完全等价。

> **别指望 clippy 替你发现多余的生命周期。** 直觉上会以为把 `'a` 留着会挨一条
> `needless_lifetimes`，实测并不会：那条 lint 只在它有十足把握的情形下出声（比如只有一个
> 输入生命周期），轮到「多个输入引用、靠 `&self` 规则省略」它就不管了。所以 `notebook`
> 里这一处既不需要 `#[allow(clippy::needless_lifetimes)]`，也别把「clippy 没报」当成
> 「这个标注是必要的」。

**含义**：返回的 `&Entry` 能活多久，取决于 `Notebook` 借给你多久，**不是**取决于查找键
`title` 活多久。所以调用方可以这么写：

```rust
let key = String::from("hello");
let entry = nb.get(&key).unwrap(); // 借用挂在 nb 手上
drop(key); // 键死了也不影响 entry
println!("{}", entry.title); // 依然能用
```

这正是测试 `returned_borrow_outlives_the_lookup_key` 想说明的事。

## 本仓库里的体现

和上篇一样，代码就是例子本身——`notebook` 三个方法凑成一组对照：

```rust
fn get<'a>(&'a self, title: &str) -> Option<&'a Entry>   // 借出去，活多久由 &self 决定
fn get_mut(&mut self, title: &str) -> Option<&mut Entry> // 可变借用，同理
fn remove(&mut self, title: &str) -> Option<Entry>       // 不借，直接把所有权还回来
```

`get` 和 `remove` 正好是一对相反操作：`get` **借出去**，能借多久由 `&self` 的生命周期约束；
`remove` 干脆**把所有权交还**给调用方，返回的是货真价实的 `Entry`，想用多久用多久。
那条界线就在这里：**借，就要受生命周期约束；干脆交出所有权，就解放了。**

## 坑

**不能返回对局部变量的引用。** 最常见的生命周期报错长这样：

```rust
fn bad() -> &str {
    let s = String::from("hi");
    &s // ERROR: returns a value referencing data owned by the current function
}
```

`s` 在函数结束时就被 `drop` 了，返回的引用会悬垂。编译器要求返回的借用必须挂在某个仍存活的
输入引用（或 `self`）上。改法三选一：把数据换成参数（借外面给的）、把所有权交出去
（`-> String`）、或干脆别在函数里新造要被返回的数据。核心就一句：**别把「自己临时造的数据」
的引用往回带。**

**两个 `&self` 可以共存，`&self` 和 `&mut self` 不能。** 借用一个对象时，要么读很多次
（很多 `&`），要么写一次（一个 `&mut`），二者不能同时出现。

**但「同时」指的是什么？——看借用最后一次被使用的位置，不是它被声明的位置。** 这条规则叫
NLL（non-lexical lifetimes），是这一章最容易想当然的地方。下面两段只差最后一行，
结果一个编不过、一个编得过：

```rust
// ❌ 编译失败：E0502
let view = nb.get("hello");     // &self 借用从这里开始
let edit = nb.get_mut("hello"); // ERROR: cannot borrow `nb` as mutable
println!("{view:?}");           // ← 正是这一行让上面那个借用一直活到现在
```

```rust
// ✅ 编译通过
let view = nb.get("hello");
println!("{view:?}");           // view 最后一次被使用，借用到此为止
let edit = nb.get_mut("hello"); // 于是这里要 &mut self 毫无问题
```

**光「声明了一个 `&self` 借用」并不构成冲突**，后面还要用它才构成。这也是为什么很多借用
报错的修法是「把那次使用挪上去」或「加一对花括号缩小作用域」——你不是在骗编译器，
你是真的让借用早点结束了。

> 这一课本仓库是踩出来的：`notebook` 里曾有一条注释写着「取消注释这行会编译失败」，
> 而实际取消之后照样编译通过——因为那个借用之后再没被用过。现在两半都由工具钉住了：
> 编不过的那半写成 `Notebook::get_mut` 文档里的 ` ```compile_fail,E0502 ` doctest，
> 编得过的那半是单元测试 `borrow_ends_at_last_use`。**「这样写会报错」这种话，
> 要么让编译器替你验证，要么别写**（做法见[笔记 10](./10-testing.md#compile_fail把报错断言交给编译器)）。

## 延伸阅读

- [The Book ch04-02 — References and Borrowing（生命周期的起步）](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [The Book ch10-03 — Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [The Rust Reference — Lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html)