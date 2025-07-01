# Geektime Rust 语言训练营


### 安装 pre-commit

pre-commit 是一个代码检查工具，可以在提交代码前进行代码检查。

```bash
pipx install pre-commit
```

安装成功后运行 `pre-commit install` 即可。

### 安装 Cargo deny

Cargo deny 是一个 Cargo 插件，可以用于检查依赖的安全性。

```bash
cargo install --locked cargo-deny
```

### 安装 typos

typos 是一个拼写检查工具。

```bash
cargo install typos-cli
```

### 安装 git cliff

git cliff 是一个生成 changelog 的工具。

```bash
cargo install git-cliff
```

### 安装 cargo nextest

cargo nextest 是一个 Rust 增强测试工具。

```bash
cargo nextest run -- test_should_work
```

```bash
cargo install cargo-nextest --locked
```

// auto reload
```bash
cargo install cargo-watch systemfd
```

```sh
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

```bash
cargo install sqlx-cli
```

## docker 启动 postgres

```bash
docker run -d --name giraffe-postgres \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=chat \
  -p 5432:5432 \
  postgres:latest
```

## pgcli 使用

### pgcli 连接 postgres

```bash
//pglic chat //连接本地数据库
pgcli postgres://postgres:password@localhost:5432/chat
```

```bash
\dt // 查看表
\d messages // 查看表结构
\d+ messages // 查看表详细信息
\d+ messages // 查看表详细信息
select * from _sqlx_migrations; // 查看迁移历史
```

## sqlx-cli 使用

### sqlx-cli 创建数据库

本地环境配置 DATABASE_URL 后，执行以下命令创建数据库，例如初始化本地数据库为 chat

```bash
// 例如初始化本地数据库为 chat
DATABASE_URL=postgres://postgres:password@localhost:5432/chat
```

### 创建数据库

```bash
sqlx database create
# 或指定自定义名称（需先连接到默认数据库）
DATABASE_URL=postgres://postgres:password@localhost:5432/chat sqlx database create
```

```bash
sqlx database drop
# 或指定自定义名称（需先连接到默认数据库）
DATABASE_URL=postgres://postgres:password@localhost:5432/chat sqlx database drop
```

### 初始化数据库

```bash
sqlx migrate add init
```

### 迁移数据库

```bash
sqlx migrate run
```

### 回滚迁移

```bash
sqlx migrate revert
```

### 证书

生成测试证书

```
openssl genpkey -algorithm Ed25519 -out chat_server/fixtures/encoding.pem
```

```
openssl pkey -in chat_server/fixtures/encoding.pem -pubout -out chat_server/fixtures/decoding.pem
```
