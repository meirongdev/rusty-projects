# 03 所有权与借用

> Rust 最核心、也是唯一没有先例可参照的概念。它让内存安全在**编译期**就得到保证，不需要垃圾回收。
> 对应项目：[`guessing_game`](../guessing_game)（零散用到）、[`notebook`](../notebook)（专门练这个）

## 三条规则

1. 每个值都有一个**所有者**（owner）；
2. 同一时刻只能有一个所有者；
3. 所有者离开作用域时，值被自动释放（`drop`）。

所以 Rust 既不需要手动 `free`，也不需要 GC——释放时机由作用域静态决定。

## 移动（move）

把一个值赋给另一个变量，或者传进函数，会**转移所有权**：

```rust
let a = String::new();
let b = a;          // 所有权移动给 b
println!("{a}");    // ❌ borrow of moved value: `a`
```

对 `i32`、`u32` 这类实现了 `Copy` 的类型则是复制，原变量仍然可用。所以下面这行传 `u32` 完全不用操心：

```rust
play(secret_number, &mut io::stdin().lock());   // secret_number: u32，按位复制
```

## 借用（borrow）

不想转移所有权，就**借**：

- `&T`：不可变借用，只能读；
- `&mut T`：可变借用，可以写。

**借用规则**：同一时刻，要么有任意多个 `&T`，要么只有一个 `&mut T`，二者不能共存。这一条从根上消灭了数据竞争。

方法上的 `&self` / `&mut self` 是同一回事——它们只是 `self: &Self` / `self: &mut Self` 的简写，
所以「拿值还是拿借用」的判断标准一模一样，详见[笔记 05](./05-structs-and-methods.md#self-的三种写法就是参数的三种拿法)。

本仓库里四种借用都出现了：

```rust
input.read_line(&mut line)          // &mut String：把输入写进 line，line 的所有权还在原处
guess.cmp(&secret_number)           // &u32：只读比较
fn parse_guess(input: &str)         // &str：借一段字符串来看，不拿走
fn play(.., input: &mut impl BufRead)  // &mut：play 要从 reader 里读，会推进它的位置
```

`read_line(&mut line)` 是理解借用最好的第一个实战例子：函数要往 `line` 里写内容，所以借的是可变引用；但 `line` 的所有权始终在调用方，函数返回后调用方接着用它。

## 动手体会

编译器的报错是这一章最好的老师。挑一个改，然后读报错：

- 把 `read_line(&mut line)` 的 `&mut` 去掉；
- 把 `fn parse_guess(input: &str)` 改成 `fn parse_guess(input: String)`，看调用方 `parse_guess(&line)` 会怎样；
- 在 `play` 里对同一个 `line` 同时持有一个 `&` 和一个 `&mut`。

每一条报错都会告诉你违反了上面哪条规则，还经常直接给出改法。

## 延伸阅读

- [The Book ch04-01 — What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html)
- [The Book ch04-02 — References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
