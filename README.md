## 安装sqlx命令行工具
```
cargo install sqlx-cli
```

## 创建sql迁移文件
```
sqlx sqlx migrate add create_oncall_table
```

## 设置环境变量
```
export DATABASE_URL="sqlite:call_phone.db"
```

## 生产sqlx迁移
```
cargo sqlx prepare
```


## 生产运行 
toml 中添加 
```
[[bin]]
name = "callme_server"
path = "src/main.rs"
```

然后 
```
cargo build --release
```


# mac arm上构建x86_64-unknown-linux-gun生产包
1. 安装交叉编译工具链：
```
brew install x86_64-unknown-linux-gnu
```

2. 为目标平台交叉编译 OpenSSL：
```
   git clone https://github.com/openssl/openssl.git
   cd openssl
   ./Configure linux-x86_64 no-shared --prefix=/usr/local/x86_64-linux-openssl --cross-compile-prefix=x86_64-unknown-linux-gnu-
   make
   make install
```
3. 设置环境变量：

```
   export OPENSSL_DIR=/usr/local/x86_64-linux-openssl
   export OPENSSL_STATIC=1
   export PKG_CONFIG_ALLOW_CROSS=1
   export PKG_CONFIG_PATH=/usr/local/x86_64-linux-openssl/lib/pkgconfig
   ```


4. 配置 Cargo：

在项目根目录创建或编辑 .cargo/config.toml：

```
   [target.x86_64-unknown-linux-gnu]
   linker = "x86_64-unknown-linux-gnu-gcc"
```

5. 添加目标：
```
   rustup target add x86_64-unknown-linux-gnu
```

6. 尝试编译：

```
   cargo build --target x86_64-unknown-linux-gnu --release
```

