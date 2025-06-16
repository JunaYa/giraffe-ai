## 错误处理

每个 crate 集中使用 errors.rs 定义错误。如果是 lib crate 则使用 thiserror，如果是 bin crate 则使用 anyhow。

这是 Rust 生态中错误处理的**最佳实践**，原因如下：

---

## 为什么每个 crate 集中用 `errors.rs` 定义错误？

- **集中管理**：所有错误类型和相关实现都放在 `errors.rs`，方便查找、维护和扩展。
- **清晰结构**：项目结构更清晰，避免错误类型分散在各个模块，提升可读性。
- **统一接口**：便于统一实现 `From`、`Display`、`Error` 等 trait，减少重复代码。

## 具体做法

### 1. Library Crate（lib crate）推荐用 [`thiserror`](https://docs.rs/thiserror/latest/thiserror/)

- 在 `src/errors.rs` 里用 `thiserror` 定义一个 `pub enum Error`，枚举所有可能的错误。
- 其他模块只需 `use crate::errors::Error;`，并通过 `Result<T, Error>` 返回错误。
- 这样，库的用户可以直接处理、匹配这些错误类型。
- **目的**：库的主要职责是为其他代码提供 API。
- **需求**：库需要定义**清晰、可区分、可枚举**的错误类型，方便用户有针对性地处理错误。
- **优势**：
  - `thiserror` 让你用简单的宏定义自定义错误类型（通常是 `enum`），并自动实现 `std::error::Error`。
  - 这样，库的用户可以**精确匹配**错误类型，做细粒度的错误处理。
  - 错误类型是**透明的**，用户可以根据需要解包、匹配、处理。

**示例**（`src/errors.rs`）：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    // 其他错误...
}
```

**在库中使用**：

```rust
use crate::errors::Error;

fn do_something() -> Result<(), Error> {
    // ...
}
```

---

### 2. Binary Crates（可执行程序）推荐用 [`anyhow`](https://docs.rs/anyhow/latest/anyhow/)

- 在 `src/errors.rs` 里可以定义一些**辅助的错误类型**或**错误处理工具函数**，但主流程直接用 `anyhow::Result<T>`。
- 如果有自定义错误类型，也可以用 `thiserror`，但最终都可以用 `anyhow::Error` 封装。
- 主程序只需关心“有没有错”，不关心具体类型，直接 `?` 传播，最后统一处理。
- **目的**：二进制 crate 主要是**最终应用**，通常只需要**打印错误并退出**，不需要区分错误类型。
- **需求**：快速、方便地传播和报告各种错误，**无需定义复杂的错误类型**。
- **优势**：
  - `anyhow` 提供了 `anyhow::Error`，可以**封装任意错误类型**，支持链式传播（`?` 操作符）。
  - 适合“遇到错误就 bubble up，最后统一处理”的场景。
  - 错误信息自动带有上下文，便于调试和日志记录。

**示例**（`src/errors.rs`，可选）：

```rust
// 可以定义一些自定义错误，但主流程用 anyhow::Result
```

```rust
use anyhow::{Result, Context};

fn main() -> Result<()> {
    do_something().context("failed to do something")?;
    Ok(())
}
```

---

### 总结

- **每个 crate 都有自己的 `errors.rs`，集中定义和管理错误。**
- **lib crate** 用 `thiserror`，暴露结构化错误类型，方便库用户处理。
- **bin crate** 用 `anyhow`，简化错误传播和处理，主流程只关心有没有错。
- **库 crate**：用 `thiserror`，暴露**结构化、可枚举**的错误类型，方便用户处理。
- **二进制 crate**：用 `anyhow`，**快速传播和统一处理**各种错误，简化代码。

这样做让项目结构清晰、易维护，可以兼顾**灵活性**（库）和**便利性**（应用），是 Rust 社区广泛认可的实践。
