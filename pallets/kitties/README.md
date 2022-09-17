# pallet-kitties

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
cargo test -p pallet-kitties
```

交互：

1、打开https://polkadot.js.org/apps/  

2、连接本地网络，依次选择DEVELOPMENT，127.0.0.1:9944

3、找到“开发者”>“交易”>“kittiesModule”，选择相应的方法进行交易

4、找到“链状态”>“kittiesModule”>相应的存储项，可以查询存储值


链的Metadata：

通过metadata可以看到整条链的数据类型、结构
怎么导出metadata？
1、安装subxt-cli
```
cargo install subxt-cli
```
安装完成后，保存位置在：~/.cargo/bin/subxt

2、导出metadata:
```
subxt metadata --url http://127.0.0.1:9933 --format json > metadata.json
```
默认是9933端口，如果不行，可以查看节点启动时的信息，如下：
```
2022-09-08 17:26:49 Running JSON-RPC HTTP server: addr=127.0.0.1:9933, allowed origins=None
```

参考：https://docs.substrate.io/reference/command-line-tools/subxt/


pallet通用模板(lib.rs)：
```
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

    /// pallet配置
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

	}

    /// pallet存储
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    /// pallet事件
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        
	}

    /// pallet错误信息
	#[pallet::error]
	pub enum Error<T> {
        
	}

    /// pallet可调用方法
	#[pallet::call]
	impl<T: Config> Pallet<T> {

	}
}

```
