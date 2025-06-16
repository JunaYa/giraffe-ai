## Best practices for using sqlx

在使用 `sqlx` 进行数据库操作时，为写入和读取的数据定义合适的 Rust 类型，并且通常会用到 `FromRow` trait。

---

### 1. 为什么要定义合适的类型？

- **类型安全**：Rust 是强类型语言，定义结构体（struct）可以确保你从数据库读取的数据和写入的数据类型一致，避免类型错误。
- **易于维护**：结构体让代码更清晰，便于维护和扩展。

---

### 2. `FromRow` 的作用

- `FromRow` trait 允许你将一行数据库查询结果自动转换为你的结构体实例。
- 只要你的结构体字段和 SQL 查询返回的列一一对应（名字和类型都兼容），`sqlx` 就能自动完成转换。

#### 例子

```rust
use sqlx::FromRow;

#[derive(Debug, FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
}
```

---

### 3. 使用 `sqlx::query_as`（而不是 `query!` 宏）

- `sqlx::query_as` 允许你将查询结果直接映射为某个结构体类型（只要实现了 `FromRow`）。
- 不用 `query!` 宏的好处是：更灵活，不依赖编译时的数据库连接，也更适合动态 SQL。

#### 例子：读取数据

```rust
let user: User = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
    .bind(1)
    .fetch_one(&pool)
    .await?;
```

#### 例子：写入数据

写入时通常用 `query`，但你可以用结构体的字段来绑定参数：

```rust
let user = User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() };

sqlx::query("INSERT INTO users (id, name, email) VALUES (?, ?, ?)")
    .bind(user.id)
    .bind(user.name)
    .bind(user.email)
    .execute(&pool)
    .await?;
```

参数绑定用 .bind()，防止 SQL 注入。

---

### 4. 数据库连接池的管理

推荐用 sqlx::Pool 并通过 axum::extract::State 注入到 handler。

```rust
use axum::{Router, extract::State, routing::get};
use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost/db")
        .await
        .unwrap();

    let app = Router::new()
        .route("/users/:id", get(get_user))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

----
### 5. 使用 sqlx-db-tester 进行单元测试

```rust

```

---
### 6. 不要在 handler 里 clone pool，每个请求自动获取连接。


---

### 总结

- **读取数据**：定义结构体 + 实现/派生 `FromRow`，用 `query_as` 自动映射。
- **写入数据**：用结构体字段绑定参数，通常用 `query`。
- **不用 `query!` 宏**：更灵活，适合动态 SQL 和无编译时数据库连接的场景。

如需具体代码示例或有其他细节问题，欢迎继续提问！
