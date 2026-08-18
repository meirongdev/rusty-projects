//! 集成测试：站在**使用者**的角度用 `notebook`。
//!
//! 和 `src/lib.rs` 里的单元测试有两点根本区别（见 docs/09）：
//!
//! 1. `tests/` 下的每个文件都是**独立的 crate**，只能通过 `use notebook::...`
//!    引入，因此只碰得到 `pub` 的东西——私有的 `entries` 字段在这里根本不存在。
//! 2. 于是它测的是「公开 API 够不够用、有没有被无意收窄」，而不是内部实现对不对。
//!    这个文件能编译过本身就是一个断言。

use notebook::{Entry, Notebook};

/// 把 README 里那段交互用纯 API 走一遍：只用公开方法就能完成全部增删改查。
#[test]
fn a_full_session_through_the_public_api_only() {
    let mut nb = Notebook::new();
    assert!(nb.is_empty());

    nb.add("hello".to_string(), "World, hello!".to_string());
    nb.add("rust".to_string(), "Ownership is the core".to_string());
    assert_eq!(nb.len(), 2);
    assert_eq!(nb.list(), vec!["hello", "rust"]);

    assert!(nb.edit_body("rust", "Borrowing too".to_string()));
    assert_eq!(nb.get("rust").unwrap().body, "Borrowing too");

    let taken = nb.remove("hello").expect("刚加进去的");
    assert_eq!(taken.body, "World, hello!");
    assert_eq!(nb.list(), vec!["rust"]);
}

/// 所有权最直观的一次演示：`remove` 交还的 `Entry` 比它出身的 `Notebook`
/// **活得更久**。
///
/// 换成 `get` 返回的借用，这段代码根本编译不过——借用不能比被借的东西活得久。
/// 「借，就要受生命周期约束；干脆交出所有权，就解放了」（docs/10）在这里
/// 是一段能跑的代码，而不是一句话。
#[test]
fn removed_entry_outlives_the_notebook_it_came_from() {
    let mut entry: Entry = {
        let mut nb = Notebook::new();
        nb.add("hello".to_string(), "World, hello!".to_string());
        nb.remove("hello").expect("刚加进去的")
        // nb 在这里离开作用域、被 drop——entry 的所有权已经出来了，不受影响
    };

    entry.body.push_str(" (mine now)");
    assert_eq!(entry.title, "hello");
    assert_eq!(entry.body, "World, hello! (mine now)");
}

/// `Entry` 的字段是 `pub` 的，所以拿到 `&mut Entry` 的使用者可以直接改写。
#[test]
fn get_mut_lets_a_caller_edit_in_place() {
    let mut nb = Notebook::new();
    nb.add("todo".to_string(), "write a test".to_string());

    let entry = nb.get_mut("todo").expect("刚加进去的");
    entry.body = "wrote it".to_string();

    assert_eq!(nb.get("todo").unwrap().body, "wrote it");
}
