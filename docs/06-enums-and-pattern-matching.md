# 06 枚举与模式匹配

> Rust 的枚举能**携带数据**，这让它比其他语言的 enum 强大得多；`match` 则强制你把每种可能都处理掉。
> 对应项目：[`guessing_game`](../guessing_game)

## 枚举可以携带数据

变体有三种形态，本仓库里都出现了：

```rust
enum GuessError {
    NotANumber,          // 单元变体：只是一个标记
    OutOfRange(u32),     // 元组变体：把「越界的那个数」一起带上
}

enum GameOutcome {
    Won { attempts: u32 },   // 结构体变体：带具名字段
    OutOfGuesses,
    InputClosed,
}
```

`OutOfRange(u32)` 带上了具体数值，调用方就能给出更精确的提示；`Won { attempts }` 让「第几次猜中的」变成结果的一部分。

后两种形态的语法和结构体是共用的（元组结构体、具名字段），见[笔记 05](./05-structs-and-methods.md#结构体把几个值捆成一个类型)。

**用带数据的枚举而不是字符串来表示错误或状态**，好处是调用方能对它做模式匹配，测试里也能精确断言「是哪一种」，而不是去比对提示文案：

```rust
assert_eq!(parse_guess("101"), Err(GuessError::OutOfRange(101)));
assert_eq!(play(42, &mut input), GameOutcome::Won { attempts: 1 });
```

> 标准库里最重要的两个枚举也是这个套路：`Option<T>`（`Some(T)` / `None`，用来表达「可能没有值」，取代其他语言的 `null`）和 `Result<T, E>`（见[笔记 07](./07-error-handling.md)）。

## `match`：穷尽的多分支

```rust
match guess.cmp(&secret_number) {
    Ordering::Less => println!("{guess} is too small!"),
    Ordering::Greater => println!("{guess} is too big!"),
    Ordering::Equal => { /* ... */ }
}
```

`cmp` 返回标准库的 `Ordering` 枚举（`Less` / `Greater` / `Equal`），配合 `match` 就是「枚举 + 模式匹配」这一 Rust 核心组合的典型示范。

匹配时可以顺手把数据**解构**出来：

```rust
match self {
    GuessError::NotANumber => write!(f, "That's not a number. ..."),
    GuessError::OutOfRange(number) => write!(f, "{number} is outside the ... range."),
    //                     ^^^^^^ 绑定到变体里携带的值
}
```

**`match` 必须穷尽所有可能。** 给 `GuessError` 加一个新变体后，编译器会立刻指出所有没覆盖到的 `match`——这是 Rust 帮你重构的方式。实在不想逐个列举时可以用 `_ => ...` 兜底，但在错误类型上通常不该这么写，否则就丧失了上面这层保护。

## `match` 是表达式

它有值，所以能整体赋给变量：

```rust
let guess = match parse_guess(&line) {
    Ok(number) => number,
    Err(error) => {
        println!("{error} Try again.");
        continue;
    }
};
```

要求是**每个分支的类型必须一致**。这里第一个分支给出 `u32`，第二个分支却是 `continue`——之所以能编译，是因为 `continue`、`break`、`return`、`panic!` 的类型是 `!`（never type，「永远不会返回」），它能适配任意类型。

## 延伸阅读

- [The Book ch06 — Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [The Book ch06-02 — The `match` Control Flow Construct](https://doc.rust-lang.org/book/ch06-02-match.html)
- [The Book ch19 — Patterns and Matching](https://doc.rust-lang.org/book/ch19-00-patterns.html)
