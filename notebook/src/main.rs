//! 一个极薄的命令行外壳：读一条命令，调 lib 里的 `Notebook`，打印结果。
//!
//! 所有真正的逻辑（增删改查）都在 lib 里，I/O 只留在这里——这正是 docs/09 讲的
//! 「把不确定性关在外面，让逻辑可测」的体现。所有权行为都要去 lib.rs 里看；
//! 这里只演示一件事：每个命令怎么拿数据（借 / 移动）由 lib 的签名决定，
//! 而这个文件里的每一处代理由那套签名直接决定。

use std::fmt;
use std::io::{self, BufRead, Write};

use notebook::Notebook;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    // 两个不确定性——从哪读、往哪写——都在 main 里接上真实世界，
    // run 自己一个都不知道。测试于是可以喂字节、收字节。
    if let Err(error) = run(stdin.lock(), &mut stdout) {
        eprintln!("write error: {error}");
    }
}

/// REPL 主循环。输入和输出都是参数，所以整段交互逻辑都能被测试：
/// 真实运行接 stdin / stdout，测试接 `&[u8]` 和 `Vec<u8>`。
///
/// 两个参数的**所有权待遇不一样**，而且都不是随便选的——签名是被标准库的
/// 签名逼出来的：
///
/// - `input: impl BufRead` 按值拿走，因为下面用的 `BufRead::lines(self)`
///   会**消耗**掉 reader；guessing_game 的 `play` 用的是 `read_line(&mut self, ..)`，
///   只需要 `&mut impl BufRead` 就够，reader 的所有权还留在调用方手里。
/// - `out: &mut impl Write` 只借，因为 `Write` 的方法都只要 `&mut self`；
///   main 那边还要继续持有 stdout 的锁，不能被拿走。
///
/// 「参数该拿值还是拿借用，看被调用方最少需要什么」——这就是 docs/03 那句
/// 「签名就是合同」在真实代码里的样子。
fn run(input: impl BufRead, out: &mut impl Write) -> io::Result<()> {
    let mut nb = Notebook::new();
    print_help(out)?;

    for line in input.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                // 这里也写进 out（而不是 stderr），是为了「程序说过的每句话都能被
                // 断言」。真实的 CLI 通常会把错误分到 stderr，代价是那部分输出
                // 测试就看不到了——这是取舍，不是定论。
                writeln!(out, "read error: {error}")?;
                break;
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_command(line) {
            Ok(Command::Add(title, body)) => {
                // title / body 是「移动」进 add 的：所有权交给 Notebook 之后
                // body 就不能再用了，所以我们只 clone 一个 title 用来显示。
                nb.add(title.clone(), body);
                writeln!(out, "added: {title}")?;
            }
            Ok(Command::List) => {
                if nb.is_empty() {
                    writeln!(out, "(empty notebook)")?;
                } else {
                    // list() 返回的是 `&str` 借用，遍历时没有任何 clone。
                    for title in nb.list() {
                        writeln!(out, "  - {title}")?;
                    }
                }
            }
            Ok(Command::Get(title)) => match nb.get(&title) {
                Some(entry) => {
                    writeln!(out, "{}", entry.title)?;
                    writeln!(out, "  {}", entry.body)?;
                }
                None => writeln!(out, "not found: {title}")?,
            },
            Ok(Command::Edit(title, body)) => {
                if nb.edit_body(&title, body) {
                    writeln!(out, "edited: {title}")?;
                } else {
                    writeln!(out, "not found: {title}")?;
                }
            }
            Ok(Command::Delete(title)) => match nb.remove(&title) {
                // remove 把 Entry 的所有权交回给我们：现在 body 归我们所有，
                // 可以放心打印出来。
                Some(entry) => writeln!(out, "deleted: {title} ({})", entry.body)?,
                None => writeln!(out, "not found: {title}")?,
            },
            Ok(Command::Quit) => {
                writeln!(out, "bye!")?;
                break;
            }
            Ok(Command::Help) => print_help(out)?,
            Err(error) => writeln!(out, "{error}")?,
        }
    }

    Ok(())
}

/// derive 了 `Debug` + `PartialEq`，测试里才能直接 `assert_eq!` 整个命令，
/// 而不用退回 `matches!` 加一串条件（前提见 docs/07）。
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Add(String, String),
    List,
    Get(String),
    Edit(String, String),
    Delete(String),
    Quit,
    Help,
}

/// 每条带参数的命令的用法。`print_help` 和「缺参数」的报错共用同一份字面量，
/// 免得改了帮助文案却忘了改报错（docs/02 里「把魔法值提成常量」的同一招）。
const USAGE_ADD: &str = "add <title> <body...>";
const USAGE_GET: &str = "get <title>";
const USAGE_EDIT: &str = "edit <title> <body...>";
const USAGE_DELETE: &str = "delete <title>";

/// 一行输入没能变成命令的两种原因。
///
/// 做成枚举而不是直接返回一句字符串，理由和 guessing_game 的 `GuessError`
/// 完全一样（见 docs/06）：调用方能 match，测试能精确断言是哪一种。
#[derive(Debug, PartialEq, Eq)]
enum ParseError {
    /// 第一个词不是任何已知命令。
    UnknownCommand(String),
    /// 命令认识，但参数不够。携带的是这条命令的用法，好直接告诉用户怎么写。
    MissingArgument(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownCommand(word) => {
                write!(f, "unknown command: {word} - type 'help'")
            }
            ParseError::MissingArgument(usage) => write!(f, "usage: {usage}"),
        }
    }
}

/// 取第一个参数（标题）。没有就报出这条命令的用法，而不是含糊的「未知命令」。
fn title_of(rest: &[&str], usage: &'static str) -> Result<String, ParseError> {
    rest.first()
        .map(|word| (*word).to_string())
        .ok_or(ParseError::MissingArgument(usage))
}

/// 正文 = 标题之后的所有词拼回去。一个词都没有也算缺参数——否则 `edit x`
/// 会静默把正文清空，`add x` 会悄悄存进一条空笔记。
fn body_of(rest: &[&str], usage: &'static str) -> Result<String, ParseError> {
    let body = rest.get(1..).unwrap_or_default().join(" ");
    if body.is_empty() {
        Err(ParseError::MissingArgument(usage))
    } else {
        Ok(body)
    }
}

/// 把一行输入解析成命令。标题是第一个词（不含空格），正文是剩下的所有词。
fn parse_command(line: &str) -> Result<Command, ParseError> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let Some((head, rest)) = words.split_first() else {
        // 只有空白的行。run 会先跳过空行，所以正常路径走不到这里。
        return Err(ParseError::UnknownCommand(String::new()));
    };

    match head.to_ascii_lowercase().as_str() {
        "add" => Ok(Command::Add(
            title_of(rest, USAGE_ADD)?,
            body_of(rest, USAGE_ADD)?,
        )),
        "edit" => Ok(Command::Edit(
            title_of(rest, USAGE_EDIT)?,
            body_of(rest, USAGE_EDIT)?,
        )),
        "get" => Ok(Command::Get(title_of(rest, USAGE_GET)?)),
        "delete" | "rm" => Ok(Command::Delete(title_of(rest, USAGE_DELETE)?)),
        "list" | "ls" => Ok(Command::List),
        "quit" | "exit" | "q" => Ok(Command::Quit),
        "help" | "?" => Ok(Command::Help),
        other => Err(ParseError::UnknownCommand(other.to_string())),
    }
}

fn print_help(out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "Commands:")?;
    for (usage, what) in [
        (USAGE_ADD, "add a note"),
        (USAGE_GET, "show one note"),
        (USAGE_EDIT, "replace the body"),
        (USAGE_DELETE, "remove a note"),
        ("list", "list all titles"),
        ("help / ?", "this help"),
        ("quit / exit", "leave"),
    ] {
        writeln!(out, "  {usage:<22}  {what}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 把一段输入喂给整个 REPL，把它打印出来的东西全收回来。
    ///
    /// `&[u8]` 实现了 `BufRead`、`Vec<u8>` 实现了 `Write`，所以不需要任何
    /// 测试框架或临时文件，标准库自带的类型就够注入了。
    fn run_with(input: &str) -> String {
        let mut out = Vec::new();
        run(input.as_bytes(), &mut out).expect("写进 Vec<u8> 不会失败");
        String::from_utf8(out).expect("输出应当是合法 UTF-8")
    }

    #[test]
    fn parses_add_with_body() {
        assert_eq!(
            parse_command("add hello World, hello!"),
            Ok(Command::Add(
                "hello".to_string(),
                "World, hello!".to_string()
            ))
        );
    }

    #[test]
    fn parses_short_aliases() {
        assert_eq!(parse_command("ls"), Ok(Command::List));
        assert_eq!(parse_command("q"), Ok(Command::Quit));
        assert_eq!(parse_command("rm x"), Ok(Command::Delete("x".to_string())));
    }

    #[test]
    fn rejects_unknown_or_empty() {
        assert_eq!(
            parse_command("sing"),
            Err(ParseError::UnknownCommand("sing".to_string()))
        );
        assert_eq!(
            parse_command(""),
            Err(ParseError::UnknownCommand(String::new()))
        );
    }

    #[test]
    fn missing_argument_is_not_an_unknown_command() {
        // 「命令认识但参数不够」和「压根不认识这个命令」是两回事，
        // 混成一句 unknown command 会把人引去检查拼写。
        assert_eq!(
            parse_command("add"),
            Err(ParseError::MissingArgument(USAGE_ADD))
        );
        assert_eq!(
            parse_command("add hello"), // 有标题没正文
            Err(ParseError::MissingArgument(USAGE_ADD))
        );
        assert_eq!(
            parse_command("get"),
            Err(ParseError::MissingArgument(USAGE_GET))
        );
        assert_eq!(
            parse_command("edit hello"),
            Err(ParseError::MissingArgument(USAGE_EDIT))
        );
        assert_eq!(
            parse_command("delete"),
            Err(ParseError::MissingArgument(USAGE_DELETE))
        );
    }

    #[test]
    fn error_message_tells_you_what_to_type() {
        assert_eq!(
            ParseError::MissingArgument(USAGE_ADD).to_string(),
            "usage: add <title> <body...>"
        );
        assert!(
            ParseError::UnknownCommand("sing".to_string())
                .to_string()
                .contains("sing")
        );
    }

    // 下面几个测的是**整段 REPL**，不是单个函数——docs/09 讲的「让结局可断言」
    // 在这里的落法是：把输出也注入进去，于是结局就是那段输出。

    #[test]
    fn add_then_list_then_get() {
        let out = run_with("add hello World, hello!\nlist\nget hello\nquit\n");
        assert!(out.contains("added: hello"), "{out}");
        assert!(out.contains("  - hello"), "{out}");
        assert!(out.contains("  World, hello!"), "{out}");
        assert!(out.ends_with("bye!\n"), "{out}");
    }

    #[test]
    fn delete_reports_the_body_it_got_ownership_of() {
        let out = run_with("add hello World, hello!\ndelete hello\nlist\n");
        assert!(out.contains("deleted: hello (World, hello!)"), "{out}");
        assert!(out.contains("(empty notebook)"), "{out}");
    }

    #[test]
    fn edit_replaces_the_body() {
        let out = run_with("add rust old\nedit rust new body\nget rust\n");
        assert!(out.contains("edited: rust"), "{out}");
        assert!(out.contains("  new body"), "{out}");
        assert!(!out.contains("  old"), "{out}");
    }

    #[test]
    fn missing_and_unknown_commands_say_different_things() {
        let out = run_with("add\nsing\n");
        assert!(out.contains("usage: add <title> <body...>"), "{out}");
        assert!(out.contains("unknown command: sing"), "{out}");
    }

    #[test]
    fn eof_ends_the_loop_without_quit() {
        // 输入喂完就结束，没有 quit。run 应当正常返回而不是空转
        //（和 guessing_game 里的 EOF 死循环坑是同一课，见 docs/04）。
        let out = run_with("add hello World, hello!\n");
        assert!(out.contains("added: hello"), "{out}");
        assert!(!out.contains("bye!"), "{out}");
    }

    #[test]
    fn blank_lines_are_skipped_silently() {
        let out = run_with("\n   \nlist\n");
        assert!(out.contains("(empty notebook)"), "{out}");
        assert!(!out.contains("unknown command"), "{out}");
    }
}
