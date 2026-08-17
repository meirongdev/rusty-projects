use rand::RngExt;
use std::cmp::Ordering;
use std::fmt;
use std::io::{self, BufRead};

/// 秘密数字的取值范围，闭区间 1..=100。
const RANGE_START: u32 = 1;
const RANGE_END: u32 = 100;

/// 玩家最多能猜的次数（非法输入不算）。
const MAX_GUESSES: u32 = 7;

/// 一行输入没能变成合法猜测的两种原因。
#[derive(Debug, PartialEq, Eq)]
enum GuessError {
    /// 输入压根不是 u32，比如 "abc"、空行、负数。
    NotANumber,
    /// 是个数字，但落在 1..=100 之外，比如 0 或 101。
    OutOfRange(u32),
}

impl fmt::Display for GuessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GuessError::NotANumber => write!(
                f,
                "That's not a number. Enter an integer between {RANGE_START} and {RANGE_END}."
            ),
            GuessError::OutOfRange(number) => {
                write!(
                    f,
                    "{number} is outside the {RANGE_START}..={RANGE_END} range."
                )
            }
        }
    }
}

/// 把一行原始输入变成一个合法的猜测。
///
/// 不碰 stdin、不打印任何东西，所以能被测试直接覆盖。
fn parse_guess(input: &str) -> Result<u32, GuessError> {
    // read_line 会把末尾的换行符一起交给我们，"42\n" 直接 parse 会失败。
    // Windows 上的 "42\r\n" 同样由 trim 处理掉。
    let number: u32 = input.trim().parse().map_err(|_| GuessError::NotANumber)?;

    if (RANGE_START..=RANGE_END).contains(&number) {
        Ok(number)
    } else {
        Err(GuessError::OutOfRange(number))
    }
}

/// 一局游戏的三种结局。返回它而不是只打印文案，测试才能断言「结果」。
#[derive(Debug, PartialEq, Eq)]
enum GameOutcome {
    /// 猜中了，携带实际用掉的次数。
    Won { attempts: u32 },
    /// 用光全部机会还没猜中。
    OutOfGuesses,
    /// 输入流在猜中之前就关闭了（Ctrl-D，或管道输入已喂完）。
    InputClosed,
}

/// 玩一整局猜数字。
///
/// 秘密数字由调用方传入，输入从注入的 reader 读取——两个不确定性都在函数
/// 外面解决，所以同样的输入必然得到同样的 `GameOutcome`，整局游戏因此可测。
fn play(secret_number: u32, input: &mut impl BufRead) -> GameOutcome {
    println!("Guess the number! I'm thinking of a number between {RANGE_START} and {RANGE_END}.");

    // 用 while 手动计数，而不是 `for attempt in 1..=MAX_GUESSES`：for 由迭代器
    // 驱动，循环体里的 continue 会推进迭代器，非法输入照样会消耗掉一次机会。
    let mut attempt = 1;
    while attempt <= MAX_GUESSES {
        println!("--- Attempt {attempt}/{MAX_GUESSES} ---");

        let mut line = String::new();
        // read_line 的返回值是读到的字节数，0 表示输入流已经关闭。不处理这种
        // 情况的话，下面那个 continue 会让程序在 EOF 上空转成死循环。
        if input.read_line(&mut line).expect("Failed to read line") == 0 {
            println!("\nInput closed. The secret number was {secret_number}.");
            return GameOutcome::InputClosed;
        }

        let guess = match parse_guess(&line) {
            Ok(number) => number,
            Err(error) => {
                println!("{error} Try again.");
                continue; // 不动 attempt：非法输入不消耗机会。
            }
        };

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("{guess} is too small!"),
            Ordering::Greater => println!("{guess} is too big!"),
            Ordering::Equal => {
                println!("You guessed {guess} — you win in {attempt} attempts!");
                // 用 return 而不是 break：break 之后会落到循环后面那句
                // "Out of guesses!"。
                return GameOutcome::Won { attempts: attempt };
            }
        }

        attempt += 1; // 只有真正猜过一次，才算用掉一次机会。
    }

    println!("Out of guesses! The secret number was {secret_number}.");
    GameOutcome::OutOfGuesses
}

fn main() {
    let secret_number = rand::rng().random_range(RANGE_START..=RANGE_END);
    // Stdin 本身只实现 Read，带缓冲的 BufRead 在 lock() 返回的 StdinLock 上。
    play(secret_number, &mut io::stdin().lock());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn accepts_plain_number() {
        assert_eq!(parse_guess("42"), Ok(42));
    }

    #[test]
    fn trims_newline_from_read_line() {
        // read_line 交给我们的就是这种带换行的字符串。
        assert_eq!(parse_guess("42\n"), Ok(42));
        assert_eq!(parse_guess("  7 \r\n"), Ok(7));
    }

    #[test]
    fn accepts_range_endpoints() {
        assert_eq!(parse_guess("1"), Ok(RANGE_START));
        assert_eq!(parse_guess("100"), Ok(RANGE_END));
    }

    #[test]
    fn rejects_non_numbers() {
        assert_eq!(parse_guess("abc"), Err(GuessError::NotANumber));
        assert_eq!(parse_guess(""), Err(GuessError::NotANumber));
        assert_eq!(parse_guess("3.14"), Err(GuessError::NotANumber));
        // 目标类型是 u32，解析负数会失败，所以负数也归到 NotANumber。
        assert_eq!(parse_guess("-5"), Err(GuessError::NotANumber));
    }

    #[test]
    fn rejects_out_of_range() {
        assert_eq!(parse_guess("0"), Err(GuessError::OutOfRange(0)));
        assert_eq!(parse_guess("101"), Err(GuessError::OutOfRange(101)));
    }

    #[test]
    fn error_message_guides_player() {
        // Display 实现是给玩家看的，值得用一个测试钉住关键信息。
        assert!(GuessError::NotANumber.to_string().contains("not a number"));
        assert!(GuessError::OutOfRange(101).to_string().contains("101"));
    }

    #[test]
    fn wins_on_first_guess() {
        let mut input = BufReader::new(&b"42\n"[..]);
        assert_eq!(play(42, &mut input), GameOutcome::Won { attempts: 1 });
    }

    #[test]
    fn win_reports_attempt_count() {
        let mut input = BufReader::new(&b"50\n75\n42\n"[..]);
        assert_eq!(play(42, &mut input), GameOutcome::Won { attempts: 3 });
    }

    #[test]
    fn runs_out_of_guesses() {
        // 秘密数字是 50，连猜 7 次都太小。
        let mut input = BufReader::new(&b"1\n2\n3\n4\n5\n6\n7\n"[..]);
        assert_eq!(play(50, &mut input), GameOutcome::OutOfGuesses);
    }

    #[test]
    fn invalid_input_keeps_attempt() {
        // abc 不是数字、101 越界，都不算一次猜测，第 1 次有效输入就猜中。
        let mut input = BufReader::new(&b"abc\n101\n42\n"[..]);
        assert_eq!(play(42, &mut input), GameOutcome::Won { attempts: 1 });
    }

    #[test]
    fn input_closed_mid_game() {
        // 只喂了一行，猜完再读就是 EOF。
        let mut input = BufReader::new(&b"10\n"[..]);
        assert_eq!(play(50, &mut input), GameOutcome::InputClosed);
    }
}
