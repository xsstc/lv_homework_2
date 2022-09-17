# pallet-poe

编译：
```
cargo build --release
```

运行：

```
./target/release/node-template --dev
```

测试：
```
cargo test -p pallet-poe
```

交互：

1、打开https://polkadot.js.org/apps/  

2、连接本地网络，依次选择DEVELOPMENT，127.0.0.1:9944

3、找到“开发者”>“交易”>“poeModule”，选择相应的方法进行交易

4、找到“链状态”>“poeModule”>相应的存储项，可以查询存储值
