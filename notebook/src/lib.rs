//! 一个「所有权」练习场的小笔记本。
//!
//! 这个 crate 没有 I/O、没有讲究的算法——它的意义是一个「谁持有数据始终清楚」的
//! 小例子。每一个公开方法在哪一种所有权行为上做文章，都写在方法头上，
//! 读完代码就相当于复习了 docs/03-ownership-and-borrowing.md 与
//! docs/10-lifetimes-and-more-ownership.md 两篇笔记。

/// 一条笔记。
///
/// 两个字段都是**拥有的** String：Entry 是它们唯一的所有者。
#[derive(Debug)]
pub struct Entry {
    /// 标题，用于查找。
    pub title: String,
    /// 正文。
    pub body: String,
}

/// 一个笔记本：Vec 里每一项 Entry 的唯一所有者就是这个 Notebook 自己。
#[derive(Debug, Default)]
pub struct Notebook {
    entries: Vec<Entry>,
}

impl Notebook {
    /// 空的笔记本。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 有多少条笔记。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否没有笔记。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// **移动**：把两个拥有的 String 移进一条新笔记，再移进 Vec。
    ///
    /// 调用之后，调用方手里的 title / body 都不再有效——所有权已经交给本函数，
    /// 再往下一直归 Notebook 所有。这就是「值会移动」的直观演示。
    pub fn add(&mut self, title: String, body: String) {
        self.entries.push(Entry { title, body });
    }

    /// **借用 + 生命周期**：用一个**借来的** &str 标题查找，返回一条**借来的** &Entry。
    ///
    /// 这里有两个输入引用（&self 和 &str），但返回值只可能源自 &self——生命周期
    /// 省略规则规定：当输入里有一个 &self/&mut self 时，返回的生命周期沿用它的。
    /// 换句话说，返回的 &Entry 能活多久，取决于 Notebook 借给你多久，而不是
    /// title 这个查找键。这也是「不能返回对局部变量的引用」这条报错的来源：
    /// 返回的借用必须还挂在某个输入引用上。
    #[allow(clippy::needless_lifetimes)] // 故意保留显式生命周期，方便对照省略规则
    pub fn get<'a>(&'a self, title: &str) -> Option<&'a Entry> {
        self.entries.iter().find(|entry| entry.title == title)
    }

    /// **可变借用**：&self 改成 &mut self，就变成全场唯一能改写数据的入口。
    ///
    /// 它和 get 不能同时使用——对同一个 Notebook 你只能要么有很多个 &self，
    /// 要么只有一个 &mut self。这条规则从根上杜绝了数据竞争。
    pub fn get_mut(&mut self, title: &str) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|entry| entry.title == title)
    }

    /// **移动 + 可变借用**：把一段新的、拥有的正文**移入**覆盖旧正文。
    ///
    /// 返回 true 表示改到了，false 表示没有这条笔记。
    pub fn edit_body(&mut self, title: &str, new_body: String) -> bool {
        match self.get_mut(title) {
            Some(entry) => {
                entry.body = new_body;
                true
            }
            None => false,
        }
    }

    /// **复制的引用**：返回所有标题的**借用**。
    ///
    /// &str 实现了 Copy，as_str() 只是把对那条 String 的借用平摊出去，
    /// 没有 clone 底层数据。
    pub fn list(&self) -> Vec<&str> {
        self.entries
            .iter()
            .map(|entry| entry.title.as_str())
            .collect()
    }

    /// **把所有权移回来**：从 Vec 里取出这条笔记，把 Entry **交还给调用方**。
    ///
    /// 这一瞬间，Notebook 是它的最后一个所有者，然后所有权整体移交给出参返回值。
    /// 拿到返回值之后，调用方可以自由使用、改写这个 Entry——它不再属于 Notebook。
    pub fn remove(&mut self, title: &str) -> Option<Entry> {
        let index = self.entries.iter().position(|entry| entry.title == title)?;
        Some(self.entries.remove(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Notebook {
        let mut nb = Notebook::new();
        nb.add("hello".to_string(), "World, hello!".to_string());
        nb.add("rust".to_string(), "Ownership is the core".to_string());
        nb
    }

    #[test]
    fn add_then_get_roundtrips() {
        let mut nb = Notebook::new();
        nb.add("todo".to_string(), "write a test".to_string());
        let entry = nb.get("todo").expect("should find it");
        assert_eq!(entry.title, "todo");
        assert_eq!(entry.body, "write a test");
    }

    #[test]
    fn get_missing_returns_none() {
        assert!(Notebook::new().get("nope").is_none());
    }

    #[test]
    fn list_returns_borrowed_titles_in_order() {
        let nb = sample();
        assert_eq!(nb.list(), vec!["hello", "rust"]);
        assert_eq!(nb.len(), 2);
    }

    #[test]
    fn edit_body_mutates_in_place() {
        let mut nb = sample();
        assert!(nb.edit_body("hello", "Edited!".to_string()));
        assert_eq!(nb.get("hello").unwrap().body, "Edited!");
        assert!(!nb.edit_body("missing", "x".to_string()));
    }

    #[test]
    fn remove_hands_ownership_back_and_removes_it() {
        let mut nb = sample();
        let removed = nb.remove("hello").expect("entry exists");

        // 所有权已经回到调用方手上：我们拿到了这个 Entry 本体，可以改写它。
        let mut owned = removed;
        owned.body.push_str(" (mine now)");
        assert!(owned.body.ends_with("(mine now)"));

        assert_eq!(nb.len(), 1);
        assert_eq!(nb.list(), vec!["rust"]);
        assert!(nb.get("hello").is_none());
    }

    #[test]
    fn remove_missing_returns_none() {
        assert!(sample().remove("nope").is_none());
    }

    // 生命周期教学：get 返回的引用只和 &self 绑定，不和查找键 &str 绑定。
    // 我们可以在拿到返回的 &Entry 之后，仍然把临时构造的 String 键扔掉。
    #[test]
    fn returned_borrow_outlives_the_lookup_key() {
        let nb = sample();
        let title_key = String::from("hello");
        let entry = nb.get(&title_key).unwrap(); // 借用挂在 nb 上
        drop(title_key); // 键被丢弃也没关系，entry 不受影响
        assert_eq!(entry.title, "hello");
    }

    // 借用语法教学：同一时刻不能既持有 get 的 &self 借用，又要 get_mut。
    // 把下面注释掉的那行取消注释，编译器会告诉你 borrow conflict 的具体位置。
    #[test]
    fn cannot_hold_ref_and_mut_at_once() {
        let mut nb = sample();
        let _ref_view = nb.get("hello"); // &self 借用
        // let _mut_view = nb.get_mut("hello"); // 会编译失败
        assert!(nb.get_mut("hello").is_some());
    }
}
