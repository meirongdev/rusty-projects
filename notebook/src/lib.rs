//! 一个「所有权」练习场的小笔记本。
//!
//! 这个 crate 没有 I/O、没有讲究的算法——它的意义是一个「谁持有数据始终清楚」的
//! 小例子。每一个公开方法在哪一种所有权行为上做文章，都写在方法头上，
//! 读完代码就相当于复习了 docs/03-ownership-and-borrowing.md 与
//! docs/11-lifetimes-and-more-ownership.md 两篇笔记。
//!
//! 方法头上那些 ` ```compile_fail ` 例子不是摆设：`cargo test` 会真的去编译它们，
//! 并要求编译**失败**。所有权的「这样写会报错」在这里是被工具钉住的断言，
//! 而不是一句没人验证的注释——doctest 的用法见 docs/10-testing.md。
//!
//! # Examples
//!
//! ```
//! use notebook::Notebook;
//!
//! let mut nb = Notebook::new();
//! nb.add("hello".to_string(), "World, hello!".to_string());
//!
//! assert_eq!(nb.get("hello").unwrap().body, "World, hello!"); // 借出去看一眼
//!
//! let entry = nb.remove("hello").unwrap(); // 所有权整个交回调用方
//! assert_eq!(entry.title, "hello");
//! assert!(nb.is_empty());
//! ```

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
///
/// 注意 `entries` **没有**标 `pub`——`pub struct` 只让类型本身公开，字段默认仍是私有的。
/// 于是外面只能通过下面那些方法操作笔记本，碰不到 `Vec` 本身：「标题不去重」这类规则
/// 因此有地方可守，将来换成 `HashMap` 也不会惊动任何调用方。这也正是
/// `tests/public_api.rs`「只看得见 pub 的东西」的由来（见 docs/05-structs-and-methods.md）。
///
/// # Examples
///
/// 私有字段意味着外部 crate 造不出 `Notebook`，必须走 [`Notebook::new`]：
///
/// ```compile_fail,E0451
/// let nb = notebook::Notebook { entries: Vec::new() }; // ERROR: field `entries` is private
/// ```
#[derive(Debug, Default)]
pub struct Notebook {
    entries: Vec<Entry>,
}

impl Notebook {
    /// 空的笔记本。
    ///
    /// 实现直接委托给 `#[derive(Default)]`：同一份「空是什么样」只写一处，
    /// 将来给 `entries` 换容器时也只改一个地方。
    pub fn new() -> Self {
        Self::default()
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
    ///
    /// # 标题不去重
    ///
    /// 这是一个 `Vec` 而不是 `HashMap`：重复标题会被原样追加两条，`get` /
    /// `get_mut` / `remove` 都只作用于**最先加进去的那条**。想要 map 语义
    /// 得自己在调用前判断，见 README 的扩展练习。
    ///
    /// # Examples
    ///
    /// 移动之后原变量就不能再用了。这一点由编译器盯着：
    ///
    /// ```compile_fail,E0382
    /// use notebook::Notebook;
    ///
    /// let mut nb = Notebook::new();
    /// let title = String::from("hello");
    /// nb.add(title, "World, hello!".to_string()); // title 的所有权进了 add
    /// println!("{title}");                        // ERROR: borrow of moved value
    /// ```
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
    ///
    /// 下面的 `'a` 是**手写**的，省略规则本来就能推出同样的签名——留着纯粹是为了
    /// 和规则逐条对照。（clippy 的 `needless_lifetimes` 不会提示这一处：它对
    /// 「靠 `&self` 规则省略」的情形并不 lint，所以这里不需要任何 `#[allow]`。）
    ///
    /// # Examples
    ///
    /// 返回的借用挂在 `&self` 上，和查找键无关——键先死也不影响它：
    ///
    /// ```
    /// use notebook::Notebook;
    ///
    /// let mut nb = Notebook::new();
    /// nb.add("rust".to_string(), "Ownership is the core".to_string());
    ///
    /// let key = String::from("rust");
    /// let entry = nb.get(&key).unwrap(); // 借用挂在 nb 上
    /// drop(key);                         // 键被丢弃
    /// assert_eq!(entry.body, "Ownership is the core"); // entry 照样能用
    /// ```
    pub fn get<'a>(&'a self, title: &str) -> Option<&'a Entry> {
        self.entries.iter().find(|entry| entry.title == title)
    }

    /// **可变借用**：&self 改成 &mut self，就变成全场唯一能改写数据的入口。
    ///
    /// 它和 get 不能同时使用——对同一个 Notebook 你只能要么有很多个 &self，
    /// 要么只有一个 &mut self。这条规则从根上杜绝了数据竞争。
    ///
    /// # Examples
    ///
    /// **「同时」看的是借用最后一次被使用的位置，不是它声明的位置**（NLL）。
    /// 下面这段编译不过，而决定性的是最后那行 `println!`：
    ///
    /// ```compile_fail,E0502
    /// use notebook::Notebook;
    ///
    /// let mut nb = Notebook::new();
    /// nb.add("hello".to_string(), "World, hello!".to_string());
    ///
    /// let view = nb.get("hello");     // &self 借用从这里开始
    /// let edit = nb.get_mut("hello"); // ERROR[E0502]: 这里要 &mut self
    /// println!("{view:?}");           // ← 正是这一行让上面那个借用还活着
    /// ```
    ///
    /// 把最后那行 `println!` 删掉，同样的代码**就能编译通过**——`view` 之后再没
    /// 被用过，借用在 `get_mut` 之前就结束了。这一半由单元测试
    /// `borrow_ends_at_last_use` 钉住。
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
    ///
    /// # Examples
    ///
    /// ```
    /// use notebook::Notebook;
    ///
    /// let mut nb = Notebook::new();
    /// nb.add("todo".to_string(), "write a test".to_string());
    ///
    /// let mut entry = nb.remove("todo").unwrap();
    /// entry.body.push_str(" (mine now)"); // 归调用方了，随便改
    ///
    /// assert!(nb.get("todo").is_none());
    /// assert_eq!(entry.body, "write a test (mine now)");
    /// ```
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
    fn new_and_default_agree() {
        assert!(Notebook::new().is_empty());
        assert_eq!(Notebook::new().len(), Notebook::default().len());
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

    // 标题不去重：这是 Vec 而非 HashMap 的直接后果，钉一个测试免得以后
    // 有人默认它是 map 语义。
    #[test]
    fn duplicate_titles_are_kept_and_first_one_wins() {
        let mut nb = Notebook::new();
        nb.add("dup".to_string(), "first".to_string());
        nb.add("dup".to_string(), "second".to_string());

        assert_eq!(nb.len(), 2);
        assert_eq!(nb.get("dup").unwrap().body, "first");

        assert_eq!(nb.remove("dup").unwrap().body, "first");
        assert_eq!(nb.get("dup").unwrap().body, "second"); // 第二条还在
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

    // 借用语法教学，正面那一半：&self 借用在**最后一次使用**处就结束（NLL），
    // 所以下面这个 get_mut 完全合法——「声明了一个 &self 借用」本身并不冲突。
    //
    // 反面那一半（借用后面还要用，于是真的冲突）写在 `get_mut` 的
    // ```compile_fail,E0502``` doctest 里，由 cargo test 编译验证；
    // 放在这里的话，这个文件自己就编译不过了。
    #[test]
    fn borrow_ends_at_last_use() {
        let mut nb = sample();

        let view = nb.get("hello").unwrap();
        assert_eq!(view.title, "hello"); // view 最后一次被使用，借用到此为止

        let edit = nb.get_mut("hello").unwrap(); // 于是这里要 &mut self 没问题
        edit.body = "Edited!".to_string();

        assert_eq!(nb.get("hello").unwrap().body, "Edited!");
    }
}
