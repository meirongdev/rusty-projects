# 04 控制流

> 三种循环长得像，驱动方式却不同。选错会写出「注释说的和代码干的是两回事」的 bug。
> 对应项目：[`guessing_game`](../guessing_game)

## 三种循环

| 写法 | 驱动方式 | 什么时候用 |
|------|---------|-----------|
| `for x in 迭代器` | **迭代器驱动**，循环次数由迭代器决定 | 遍历集合、跑固定次数 |
| `while 条件` | **条件驱动**，什么时候推进状态由你决定 | 推进节奏不规则 |
| `loop` | 无限循环，只能靠 `break` 退出 | 退出条件在循环体中间 |

`loop` 的 `break` 还能带值返回：

```rust
let x = loop { break 5; };   // x == 5
```

## 本仓库踩过的坑：`for` + `continue`

猜数字游戏要求「最多猜 7 次，**输错不消耗机会**」。直觉写法是这样：

```rust
// ⚠️ 这段是错的
for attempt in 1..=MAX_GUESSES {
    // ...
    Err(_) => {
        println!("输入无效");
        continue;   // 想的是「重试，不消耗机会」
    }
}
```

但 **`for` 由迭代器驱动，`continue` 会推进迭代器**——输错一次照样消耗一次机会。连着输 7 个 `abc`，游戏就直接结束了。注释写的和代码干的是两回事。

想要「输错不算数」，就得自己掌握计数：

```rust
let mut attempt = 1;
while attempt <= MAX_GUESSES {
    // ...
    Err(error) => {
        println!("{error} Try again.");
        continue;          // 不动 attempt
    }
    // ...
    attempt += 1;          // 只有真正猜过一次，才算用掉一次机会
}
```

**代价：手动控制计数意味着你要自己保证循环能终止。** 本项目里如果不处理 `read_line` 返回 `0`（输入流已关闭），非法输入分支的 `continue` 就会让程序在 EOF 上空转成死循环：

```rust
if input.read_line(&mut line).expect("Failed to read line") == 0 {
    println!("\nInput closed. The secret number was {secret_number}.");
    return GameOutcome::InputClosed;
}
```

`read_line` 的返回值是「读到了多少字节」，很容易被忽略，返回 `0` 就意味着流已关闭（用户按了 Ctrl-D，或输入是管道喂进来的且已喂完）。

**把 `for` 换成 `while` 时，「循环凭什么会结束」这个问题必须重新回答一遍。**

## 范围（Range）

```rust
1..=100      // 闭区间，包含 100
1..100       // 半开区间，不含 100
(RANGE_START..=RANGE_END).contains(&number)
```

范围类型自带 `contains` 方法，比手写 `number >= 1 && number <= 100` 更清楚——clippy 也会这么建议。范围本身还是迭代器，所以能直接喂给 `for`。

## `return` 还是 `break`

猜中时本仓库用的是 `return`：

```rust
Ordering::Equal => {
    println!("You guessed {guess} — you win in {attempt} attempts!");
    return GameOutcome::Won { attempts: attempt };
}
```

用 `break` 也能跳出循环，区别是 `break` 之后会**继续执行循环后面的语句**——这里就会误打印一句 "Out of guesses!"。想从函数里直接走人就用 `return`。

## 延伸阅读

- [The Book ch03-05 — Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html)
- [The Book ch13 — Iterators and Closures](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
