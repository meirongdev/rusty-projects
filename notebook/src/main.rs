//! 一个极薄的命令行外壳：读一条命令，调 lib 里的 `Notebook`，打印结果。
//!
//! 所有真正的逻辑（增删改查）都在 lib 里，I/O 只留在这里——这正是 docs/09 讲的
//! 「把不确定性关在外面，让逻辑可测」的体现。所有权行为都要去 lib.rs 里看；
//! 这里只演示一件事：每个命令怎么拿数据（借 / 移动）由 lib 的签名决定，
//! 而这个文件里的每一处代理由那套签名直接决定。

use std::io::{self, BufRead};

use notebook::Notebook;

fn main() {
    let stdin = io::stdin();
    run(stdin.lock());
}

/// REPL 主循环。把 stdin 注入成参数，是为了让整段交互逻辑都能被测试。
fn run(input: impl BufRead) {
    let mut nb = Notebook::new();
    print_help();

    for line in input.lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("read error: {error}");
                break;
            }
        };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parse_command(line) {
            Some(Command::Add(title, body)) => {
                // title / body 是「移动」进 add 的：所有权交给 Notebook 之后
                // body 就不能再用了，所以我们只 clone 一个 title 用来显示。
                nb.add(title.clone(), body);
                println!("added: {title}");
            }
            Some(Command::List) => {
                if nb.is_empty() {
                    println!("(empty notebook)");
                } else {
                    // list() 返回的是 `&str` 借用，遍历时没有任何 clone。
                    for title in nb.list() {
                        println!("  - {title}");
                    }
                }
            }
            Some(Command::Get(title)) => match nb.get(&title) {
                Some(entry) => {
                    println!("{}", entry.title);
                    println!("  {}", entry.body);
                }
                None => println!("not found: {title}"),
            },
            Some(Command::Edit(title, body)) => {
                if nb.edit_body(&title, body) {
                    println!("edited: {title}");
                } else {
                    println!("not found: {title}");
                }
            }
            Some(Command::Delete(title)) => match nb.remove(&title) {
                // remove 把 Entry 的所有权交回给我们：现在 body 归我们所有，
                // 可以放心打印出来。
                Some(entry) => println!("deleted: {title} ({})", entry.body),
                None => println!("not found: {title}"),
            },
            Some(Command::Quit) => {
                println!("bye!");
                break;
            }
            Some(Command::Help) => print_help(),
            None => println!("unknown command - type 'help'"),
        }
    }
}

enum Command {
    Add(String, String),
    List,
    Get(String),
    Edit(String, String),
    Delete(String),
    Quit,
    Help,
}

/// 把一行输入解析成命令。标题是第一个词（不含空格），正文是剩下的所有词。
fn parse_command(line: &str) -> Option<Command> {
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.first()?.to_ascii_lowercase().as_str() {
        "add" => Some(Command::Add(
            words.get(1)?.to_string(),
            words[2..].join(" "),
        )),
        "edit" => Some(Command::Edit(
            words.get(1)?.to_string(),
            words[2..].join(" "),
        )),
        "get" => Some(Command::Get(words.get(1)?.to_string())),
        "delete" | "rm" => Some(Command::Delete(words.get(1)?.to_string())),
        "list" | "ls" => Some(Command::List),
        "quit" | "exit" | "q" => Some(Command::Quit),
        "help" | "?" => Some(Command::Help),
        _ => None,
    }
}

fn print_help() {
    for line in [
        "Commands:",
        "  add <title> <body...>   add a note",
        "  get <title>             show one note",
        "  edit <title> <body...>  replace the body",
        "  delete <title>          remove a note",
        "  list                    list all titles",
        "  help / ?                this help",
        "  quit / exit             leave",
    ] {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_add_with_body() {
        let cmd = parse_command("add hello World, hello!").unwrap();
        assert!(
            matches!(cmd, Command::Add(title, body) if title == "hello" && body == "World, hello!")
        );
    }

    #[test]
    fn parses_short_aliases() {
        assert!(matches!(parse_command("ls"), Some(Command::List)));
        assert!(matches!(parse_command("q"), Some(Command::Quit)));
    }

    #[test]
    fn rejects_unknown_or_empty() {
        assert!(parse_command("sing").is_none());
        assert!(parse_command("").is_none());
    }
}
