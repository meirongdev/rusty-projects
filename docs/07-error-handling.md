# 07 错误处理

> Rust 没有异常。可能失败的操作把失败写进**返回类型**里，编译器逼着你处理。
> 对应项目：[`guessing_game`](../guessing_game)

## `Result<T, E>`

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

它就是一个普通枚举（[笔记 06](./06-enums-and-pattern-matching.md)），没有任何魔法。`parse()`、`read_line()` 这类可能失败的操作都返回它，你必须显式决定失败时怎么办。

## 三种处理方式，从粗到细

### 1. `.expect(...)` / `.unwrap()`：直接 panic

```rust
input.read_line(&mut line).expect("Failed to read line");
```

失败就带着这句话崩掉。`unwrap()` 是不带自定义消息的版本。

什么时候可以接受：**这个错误真的没法恢复，或者继续跑下去也没意义**。这里 stdin 读取失败属于此类。`expect` 比 `unwrap` 更好，因为 panic 信息里会有你写的那句话。写库代码时则应尽量把错误传出去，让调用方决定。

### 2. `match`：分情况处理

```rust
let guess = match parse_guess(&line) {
    Ok(number) => number,
    Err(error) => {
        println!("{error} Try again.");
        continue;
    }
};
```

最啰嗦但最灵活——这里成功就往下走，失败则提示并重来。

### 3. `?`：把错误甩给调用方

```rust
let number: u32 = input.trim().parse().map_err(|_| GuessError::NotANumber)?;
```

`?` 跟在 `Result` 后面：成功就取出值继续往下走，失败就立刻把错误 `return` 给调用方。它是这段的简写：

```rust
match expr {
    Ok(v) => v,
    Err(e) => return Err(e.into()),
}
```

`?` 只能用在返回 `Result`（或 `Option`）的函数里，是 Rust 错误处理最常见的写法。

## 自定义错误类型

```rust
#[derive(Debug, PartialEq, Eq)]
enum GuessError {
    NotANumber,
    OutOfRange(u32),
}
```

为什么不直接返回 `String`？因为字符串只能被打印，不能被判断。做成枚举后：调用方可以 `match` 出具体是哪种失败并区别对待，测试可以精确断言 `Err(GuessError::OutOfRange(101))` 而不是比对提示文案，将来加一种错误时编译器还会提醒所有没覆盖到的地方。

`#[derive(...)]` 在这里的作用见[笔记 08](./08-traits.md)：`Debug` 让它能被 `{:?}` 打印（`assert_eq!` 失败时靠它），`PartialEq` 让它能用 `==` 比较（测试里要用）。

## `map_err`：转换错误类型

`parse()` 返回的是标准库的 `ParseIntError`，而函数要返回 `GuessError`，所以先转一道：

```rust
.map_err(|_| GuessError::NotANumber)?
```

`|_| GuessError::NotANumber` 是一个**闭包**，`_` 表示不关心原始错误的内容，直接换成自己的类型。转换完了 `?` 才能顺利把它甩出去。

## 「出了什么事」和「怎么说给人听」要分开

错误类型只负责描述发生了什么，展示交给 `Display`（[笔记 08](./08-traits.md)）：

```rust
println!("{error} Try again.");
```

想换文案（比如改成中文）只需改 `impl Display` 那一处，错误类型本身不动。

## 延伸阅读

- [The Book ch09 — Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [The Book ch09-02 — Recoverable Errors with `Result`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
