# Instructions

请仔细阅读 @/isolation_rules ，根据它的 rule set 体系，比如在不同的场景或条件下加载不同的 rule，通过 mermaid chart 来指导如何使用 rules 等等。请根据类似的思路帮我处理和扩充以下规则，构建一套处理大型 rust 项目的 rule set：

1. 如果项目规模不大，则使用单 crate；如果项目复杂，则拆分成多个 crate，使用 workspace 管理，每个 crate 都放入 workspace 的 dependency 中
2. crate 内部要有合理的文件设置，以功能而非类型划分文件，比如: lib.rs, models.rs, handlers.rs，而非 lib.rs, types.rs, traits.rs, impl.rs。每个文件除去 unit test 的代码行数不超过 500 行，否则将其转换成目录，然后在目录下拆分成多个文件。
3. 每个函数要遵循 DRY / SRP，函数大小不超过 150 行。
4. 每个 crate 集中使用 errors.rs 定义错误。如果是 lib crate 则使用 thiserror，如果是 bin crate 则使用 anyhow。
5. bin crate 要保持 main.rs 简洁，核心逻辑放在其他文件中，由 lib.rs 统一管理。
6. 如果使用 serde，那么遵循 serde best practice，并且使用 serde 的数据结构要 rename all CamelCase，以便生成的 json 适合前端。
7. 对于复杂的数据结构如果要能够用 new 构造，那么如果 new 的参数复杂（>=4），请引入 typed_builder，对数据结构使用 TypedBuilder，并对每个字段根据情况引入 default, default_code, 以及 setter(strip_option), setter(into), 或者 setter(strip_option, into)。比如 Option<String> 要使用 `#[builder(default, setter(strip_option, into)]`.
8. 如果需要 web framework，那么必须使用 axum，axum 必须构造 AppConfig / AppState，AppConfig 通过 arc_swap 放入 AppState 中。同时，API 和输入输出需要使用 utoipa，让 API 支持 openapi spec，并引入 utoipa swagger 支持 swagger endpoint。
9. 如果使用 sqlx，那么写入数据库和读取的数据都需要定义合适的类型，并使用 FromRow。使用 sqlx::query_as，不要使用任何 query! 宏。sqlx 相关 unit test 代码使用 sqlx-db-tester。
10. 在并发场景下，遵循 Rust 并发处理最佳实践。如果是 primitive type，使用 AtomicXXX 类型，否则如果非频繁更新，可以考虑使用 arc_swap，否则如果可以使用 dashmap，则使用 dashmap，不能使用，则可以选择 tokio 下的 Mutex 或者 RwLock。
11. unit test 必须写在和代码同一文件中，所有公开接口都需要足够正交的 unit test 来覆盖。
    整套规则写入 .cursor/rules/rust 目录，以 .cursor/rules/rust/main.mdc 为入口规则。所有规则都用 English.

请仔细阅读已有的 @/rust rule set，
@instructions.md 是我的一个 real world rust application 里面使用的所有 prompt，里面包含了一些 rust 项目的 best practice，请抽取这些 best practice 并更新 rust rule sets。

刚才更新和生成的 rule 中的示例代码包含特定的系统（比如 workflow / node），请重新审视所有的 rules，确保 rules 是尽可能 general。

我这里有一个我经常使用的 crate 的列表，请加入或者更新到 rules 中（这些 deps 根据需要引入）：

```toml
anyhow = "1.0"
async-trait = "0.1"
atomic_enum = "0.3"
axum = { version = "0.8", features = ["macros", "http2"] }
base64 = "0.22"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.0", features = ["derive"] }
dashmap = { version = "6", features = ["serde"] }
derive_more = { version = "2", features = ["full"] }
futures = "0.3"
getrandom = "0.3"
htmd = "0.2"
http = "1"
jsonpath-rust = "1"
jsonwebtoken = "9.0"
minijinja = { version = "2", features = [
"json",
"loader",
"loop_controls",
"speedups",
] }
rand = "0.8"
regex = "1"
reqwest = { version = "0.12", default-features = false, features = [
"charset",
"rustls-tls-webpki-roots",
"http2",
"json",
"cookies",
"gzip",
"brotli",
"zstd",
"deflate",
] }
schemars = { version = "0.8", features = ["chrono", "url"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
sqlx = { version = "0.8", features = [
"chrono",
"postgres",
"runtime-tokio-rustls",
"sqlite",
"time",
"uuid",
] }
thiserror = "2.0"
time = { version = "0.3", features = ["serde"] }
tokio = { version = "1.45", features = [
"macros",
"rt-multi-thread",
"signal",
"sync",
] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
typed-builder = "0.21"
url = "2.5"
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-axum = { version = "0.2" }
utoipa-swagger-ui = { version = "9", features = [
"axum",
"vendored",
], default-features = false }
uuid = { version = "1.17", features = ["v4", "serde"] }
```

@workspace.mdc 里面 workspace 的例子不好，我们应该根据系统的各个子系统来划分 crate，请更新 example

请构建 Rust CLI 项目的 rules:

1. 如果项目需要使用到 CLI，则引入 clap，使用 derive feature。
2. 如果有多个 CLI，请使用 subcommand。
3. 构建应用于 command 的 execute trait，每个 subcommand 实现该 trait，并使用 enum_dispatch 进行 dispatch

```rust
#[enum_dispatch(...)]
pub trait CommandExecutor {
    async fn execute(&self, args: &Args) -> Result<(), Error>;
}
```

4. 其他请遵循 clap 最佳实践

现在请帮我构建 protobuf / grpc rules，并更新 @main.mdc。当项目需要构建 protobuf / grpc 应用时，需要使用 prost / tonic 最新版本。一些 best practice:

1. prost / tonic 生成的代码放在 src/pb 中，注意添加 src/pb/mod.rs 引用所有生成的文件，然后在 lib.rs 中 `pub mod pb;` 进行引用。
2. 使能 format feature，并且设置 format(true)。
3. 如果生成的代码名为 src/pb/a.b.rs，请在 build.rs 中将其重命名（比如如果不存在重名风险，则 src/pb/b.rs 或者 src/pb/a/b.rs）。注意生成对应的 mod.rs
4. 为 prost/tonic 生成的每个数据结构提供对应的结构，比如 Foo，则生成 FooInner。相对于 Foo，它不包含不必要的 Option，并且包含 #[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)] 等 attribute。它也实现了 From<FooInner> for Foo 的 trait。
5. prost/tonic 生成的数据结构要实现一个 MessageSanitizer trait:

````rust
pub trait MessageSanitizer {
  type Output;
  fn sanitize(self) -> Self::Output;
}
比如 type 是 Foo，那么 Output 是 FooInner，sanitize 会处理各种 default 场景，比如 Option<Bar> default 是 None，我们转换时为 FooInner 提供 BarInner 的 default 值。
5. 对于 grpc service，先为数据结构生成同名的，输入输出更简洁的方法，再在 grpc service trait 的实现中引用这些方法。比如：
```rust
// 不要这样实现
#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}
// 使用如下实现
impl MyGreeter {
async pub fn say_hello(
        &self,
        request: HelloRequestInner,
    ) -> Result<HelloReplyInner, Error> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReplyInner {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(reply)
    }
}
// 然后在 impl Greeter for MyGreeter 中调用这个方法（需要转换输入输出）
````

通过这种方法，unit test 可以更好地测试数据结构的方法，而避免复杂的输入输出的构建。

几处修改：

1. 使用 From trait，不要额外定义 ToProtobuf。
2. 在 build.rs 中使用 pb_dir，不要使用 out_dir。
3. 使用 prost_types，不要使用 compile_well_known_types(true)
4. 不要添加 protoc_arg
5. 对 primitive type 不需要 sanitize_optional_xxx。
6. TypedBuilder 用法遵循:并对每个字段根据情况引入 default, default_code, 以及 setter(strip_option), setter(into), 或者 setter(strip_option, into)。比如 Option<String> 要使用 `#[builder(default, setter(strip_option, into)]`. 不要滥用 default。

请仔细审核 @/rust rule set，看各部分内容是否有重复，是否正交，请相应修改和重构，另外，如果还有 best practice 和 rust 最佳实践和设计模式没有写进去，请添加。

请再次仔细审视 @/rust rules set，请确保：

1. rust best practices 已涵盖
2. rust rules 是正交的，且通过 @main.mdc 可以根据不同的场景动态加载不同的 rules。
3. 目前的子目录设置是合理的
4. 核心规则 @/core 是合理的，且利于产生高质量且简洁的代码
5. @/features 规则是合理的，且利于产生高质量且简洁的代码
6. 这些 rules 无论对新项目还是老项目都能够产生高质量且简洁的代码
   如果你发现需要任何修改，请在 ./spec/ 下生成一个详细的修改计划

请仔细阅读 @/rust rule sets， 然根据下面列出的 prompt，找出 common best practice，在 specs 目录下生成一个你想如何更新 rust rules 的详细计划。注意，如果你要添加新的代码示例，请确保它是足够 common 的。如果根据以下内容你觉得需要新建一个 pingora 的 rules，请也在新的计划文件中详细说明。

请更新 @rust-rules-improvement-plan.md 里的代码足够 common，不要包含特定的内容，而且内容需要保持正交（hot-reloading 出现一次就可以了）。之后，请按照计划更新或者生成新的 rules。不必更新 performance.mdc，不必更新 api-design.mdc，也不必添加 pingora.mdc
