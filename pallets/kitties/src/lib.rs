#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{ Randomness, Currency, ReservableCurrency, ExistenceRequirement }; // 引入Randomness,Currency,ReservableCurrency

	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;        // 引入哈希包
	use sp_runtime::traits::{ AtLeast32BitUnsigned, Bounded, One };  // 引入

    /// 定义配置
	/// 模块配置接口Config（在rust中使用trait来定义接口）
	/// Config接口继承自frame_system::Config，这样Config就拥有了系统定义的数据类型，比如BlockNumber，哈希类型Hashing，AccountId
	/// #[pallet::config]为宏
	#[pallet::config]
	pub trait Config: frame_system::Config {
        /// 事件
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// 引入Randomness随机类型
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;

		/// 引入资产类型，以便支持质押
		/// 参考：substrate/frame/treasury/src/lib.rs中的定义
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		
		// 定义KittyIndex类型，要求实现执行的trait
		// Paramter 表示可以用于函数参数传递
		// AtLeast32Bit 表示转换为u32不会造成数据丢失
		// Default 表示有默认值
		// Copy 表示实现Copy方法
		// Bounded 表示包含上界和下界
		// 以后开发遇到在Runtime中定义无符号整型，可以直接复制套用
		type KittyIndex: Parameter + AtLeast32BitUnsigned + Default + Copy + Bounded + MaxEncodedLen;

		// 定义常量时，必须带上以下宏
		#[pallet::constant]
		// 获取Runtime中Kitties pallet定义的质押金额常量
		// 在创建Kitty前需要做质押，避免反复恶意创建
        type KittyStake: Get<BalanceOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// 定义Kitty ID类型
	// type KittyIndex = u32; //已移到Runtime中定义

	#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
    /// 定义Kitty的结构体
	/// 一个由16个u8类型元素组成的数组
	pub struct Kitty(pub [u8; 16]);
	
	/// 定义账号余额
	/// 参考：substrate/frame/nicks/src/lib.rs中的定义
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// 定义存储
	#[pallet::storage]
	#[pallet::getter(fn next_kitty_id)] // getter声明外部要查询存储时，可以调用next_kitty_id方法，方法名称可自定义
    /// 存储kitty最新的id，用作索引，也可以用作kitty数量总计(+1)
	pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>; // KittyIndex移到Runtime后，KittyIndex改为T::KittyIndex

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
    // 存储kitties详情  用哈希map来存储，id => Kitty结构体
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owner)]
    /// 存储kitty对应的owner  用哈希map来存储，id => AccountId
	/// 通过KittyIndex来查找Owner 
	pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;

	/// Kitty交易市场 存储正在销售的Kitty  KittyIndex => BalanceOf 即指定Kitty => 报价
    /// 如果 Option<BalanceOf<T>> 为None, 意味着该Kitty不参与销售.
    #[pallet::storage]
	#[pallet::getter(fn kitties_list_for_sales)]
	pub type KittiesShop<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<BalanceOf<T>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {

        /// 创建kitty [who, kitty_id]
		KittyCreated(T::AccountId, T::KittyIndex),

        /// 繁殖kitty [who, kitty_id, kitty]
		KittyBred(T::AccountId, T::KittyIndex, Kitty),

        /// 转让kitty [who, to, kitty]
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),

		/// 出售kitty中 [who, kitty_id, price]
		KittyInSell(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>), 
	}

	#[pallet::error]
	pub enum Error<T> {
        /// 无效的kitty_id
		InvalidKittyId,

		/// kitty_id溢出
		KittyIdOverflow,

        /// 不是kitty的主人
		NotOwner,

        /// 重复的kitty_id
		SameKittyId,

		/// 买卖Kitty时，不能自己买自己
		NoBuySelf, 
		
		/// 非卖品
		NotForSale,    
		
		/// 没有足够的余额
		NotEnoughBalance, 

	}

    /// 定义可调用函数
	/// 可调度函数允许用户与pallet交互并调用状态更改
	/// 这些函数具体化为“外部函数”，通常与交易进行比较
	/// Dispatchable 函数必须设置权重，并且必须返回 DispatchResult
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
        /// 创建Kitties
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			
            // 获取当前操作者账户
            let sender = ensure_signed(origin)?;

            // 生成kitty的DNA
			let dna = Self::random_value(&sender);

			// // 获取需要质押的金额
            // let stake_amount = T::KittyStake::get();

			// 代码封装、重构
			Self::new_kitty_with_stake(&sender, dna)?;

            // 返回OK
			Ok(().into())
		}

		#[pallet::weight(10_000)]
        /// 繁殖Kitties
		pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResultWithPostInfo {
			
            // 获取当前操作者账户
            let sender = ensure_signed(origin)?;

			// 检查父母不能是同一个kitty
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameKittyId);
            
            // 检查kitty_id是否存在且有效
			let kitty_1 = Self::get_kitty(kitty_id_1)
				.map_err(|_| Error::<T>::InvalidKittyId)?;
			let kitty_2 = Self::get_kitty(kitty_id_2)
				.map_err(|_| Error::<T>::InvalidKittyId)?;

			let dna_1 = kitty_1.0;
			let dna_2 = kitty_2.0;

			// 生成一个随机数，作为子kitty的独有基因
			let selector = Self::random_value(&sender);

            // 通过把父母的基因与子kitty的独有基因进行位与、位或，得到子Kitty的完整基因
			let mut new_dna = [0u8; 16];
			for i in 0..dna_1.len() {
				new_dna[i] = (dna_1[i] & selector[i]) | (dna_2[i] & !selector[i]);
			}

			// // 获取需要质押的金额
            // let stake_amount = T::KittyStake::get();

			// // 质押指定数量的资产，如果资产质押失败则报错
			// T::Currency::reserve(&sender, stake_amount)
			// 	.map_err(|_| Error::<T>::NotEnoughBalance)?;
			
			// // 获取一个最新的kitty_id，如果返回出错，则提示无效的ID
			// let kitty_id = Self::get_next_id()
			// 	.map_err(|_| Error::<T>::InvalidKittyId)?;
			
			// let new_kitty = Kitty(new_dna);

            // // 保存数据
			// <Kitties<T>>::insert(kitty_id, &new_kitty);
			// KittyOwner::<T>::insert(kitty_id, &sender); // 子kitty归操作者所有
			// NextKittyId::<T>::set(kitty_id + One::one());

            // // 触发事件
			// Self::deposit_event(Event::KittyCreated(sender, kitty_id));

			// 代码封装、重构
			Self::new_kitty_with_stake(&sender, new_dna)?;

            // 返回OK
			Ok(().into())
		}

		#[pallet::weight(10_000)]
        /// 转让Kitties
		pub fn transfer(origin: OriginFor<T>, kitty_id: T::KittyIndex, new_owner: T::AccountId) -> DispatchResultWithPostInfo {
			
            // 获取当前操作者账户
            let sender = ensure_signed(origin)?;

            // 检查kitty_id是否有效
			// map_err ? 可以理解成三目运算符，满足条件就返回值，不满足条件就返回错误信息
			Self::get_kitty(kitty_id)
				.map_err(|_| Error::<T>::InvalidKittyId)?;

            // 检查是否为kitty的owner
			// 只有条件为true时，才不会报后面的Error，ensure!()相当于solidity中的require()
			ensure!(Self::kitty_owner(kitty_id) == Some(sender.clone()), Error::<T>::NotOwner);

			// 获取需要质押的金额
            let stake_amount = T::KittyStake::get();

			// 新的Owner账户进行质押
            T::Currency::reserve(&new_owner, stake_amount)
                .map_err(|_| Error::<T>::NotEnoughBalance)?;

			// 旧的Owner账户解除质押
            T::Currency::unreserve(&sender, stake_amount);

            // 保存kitty的新owner 更新也使用insert，即重新插入一条新记录覆盖原来的老数据
			<KittyOwner<T>>::insert(&kitty_id, &new_owner);

            // 触发事件
			Self::deposit_event(Event::KittyTransferred(sender, new_owner, kitty_id));

            // 返回OK
			Ok(().into())
		}	
		
        #[pallet::weight(1_000)]
		/// 出售Kitty
		/// 设置一个价格，并上架到店铺，允许价格为None，如果报价为None，则该Kitty为非卖品
        pub fn sell(origin: OriginFor<T>, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>) -> DispatchResultWithPostInfo {
            
			// 获取当前操作者账户
			let seller = ensure_signed(origin)?;

			// 检查是否为kitty的owner
			// 只有条件为true时，才不会报后面的Error，ensure!()相当于solidity中的require()
			ensure!(Self::kitty_owner(kitty_id) == Some(seller.clone()), Error::<T>::NotOwner);

			// 给指定Kitty报价并上架到店铺
            KittiesShop::<T>::mutate_exists(kitty_id, |p| *p = Some(price));
            
			// 触发出售事件
            Self::deposit_event(Event::KittyInSell(seller, kitty_id, price));

            Ok(().into())
        }

		/// 购买Kitty
		/// 从店铺购买一只Kitty
        #[pallet::weight(1_000)]
        pub fn buy(origin: OriginFor<T>, kitty_id: T::KittyIndex) -> DispatchResultWithPostInfo {
            
			// 获取买家账户
			let buyer = ensure_signed(origin)?;

			// 获取卖家账户，即Kitty的Owner
            let seller = KittyOwner::<T>::get(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

			// 检查买卖双方是否为同一个人
            ensure!(Some(buyer.clone()) != Some(seller.clone()), Error::<T>::NoBuySelf);

            // 获取指定Kitty的报价，如果报价为None，则该Kitty为非卖品
            let price = KittiesShop::<T>::get(kitty_id).ok_or(Error::<T>::NotForSale)?;
            
			// 获取买家的账户余额
			let buyer_balance = T::Currency::free_balance(&buyer);
            
			// 获取需要质押的金额配置
			let stake_amount = T::KittyStake::get();
            
			// 检查买家的余额是否足够用于购买和质押
			ensure!(buyer_balance > (price + stake_amount), Error::<T>::NotEnoughBalance);

            // 买家质押
            T::Currency::reserve(&buyer, stake_amount)
                .map_err(|_| Error::<T>::NotEnoughBalance)?;

            // 卖家解除质押
			T::Currency::unreserve(&seller, stake_amount);

            // 买家支付token给卖家
			T::Currency::transfer(&buyer, &seller, price, ExistenceRequirement::KeepAlive)?;
            
			// 将Kitty从店铺下架 删除使用remove
			KittiesShop::<T>::remove(kitty_id);

            // 更新Kitty归属买家
            KittyOwner::<T>::insert(kitty_id, buyer.clone());

            // 触发转移事件
            Self::deposit_event(Event::KittyTransferred(seller, buyer, kitty_id));

            Ok(().into())
        }


	}

    /// 定义pallet的公共函数
	impl<T: Config> Pallet<T> {
		// 获取一个256位的随机数 用作Kitty的DNA
		fn random_value(sender: &T::AccountId) -> [u8; 16] {

			let payload = (
				T::Randomness::random_seed(), // 随机值，保证dna的唯一性
				&sender,
				<frame_system::Pallet::<T>>::extrinsic_index(), // //获取当前交易在区块中的index，相当于nonce
			);

			payload.using_encoded(blake2_128) // 对payload进行Scale编码，这里需要引入use sp_io::hashing::blake2_128;
		}

		// 获取下一个kitty_id
		// fn get_next_id() -> Result<T::KittyIndex, DispatchError> {
		// 	let kitty_id = Self::next_kitty_id();
		// 	if kitty_id == T::KittyIndex::max_value() {
		// 		return Err(Error::<T>::KittyIdOverflow.into());
		// 	}
		// 	Ok(kitty_id)
		// }

		// 通过id查询Kitty
		fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty),
				None => Err(()),
			}
		}

		// 质押并创建Kitty
        // 用于优化create()和breed()
		// TODO  可考虑加入Kitty的繁殖代数
        fn new_kitty_with_stake(sender: &T::AccountId, dna: [u8; 16]) -> DispatchResultWithPostInfo {

			// 获取需要质押的金额
            let stake_amount = T::KittyStake::get();

			// 质押指定数量的资产，如果资产质押失败则报错
			T::Currency::reserve(&sender, stake_amount)
				.map_err(|_| Error::<T>::NotEnoughBalance)?;

            // 获取一个最新的kitty_id，如果返回出错，则提示无效的ID
			// map_err ? 可以理解成三目运算符，满足条件就返回值，不满足条件就返回错误信息
			// let kitty_id = Self::get_next_id()
			// .map_err(|_| Error::<T>::InvalidKittyId)?;

			let kitty_id = Self::next_kitty_id();
			if kitty_id == T::KittyIndex::max_value() {
				return Err(Error::<T>::KittyIdOverflow.into());
			}
			
			let kitty = Kitty(dna);

			// 保存数据
			Kitties::<T>::insert(kitty_id, &kitty); // 保存kitty信息
			KittyOwner::<T>::insert(kitty_id, &sender); // 保存kitty的owner
			NextKittyId::<T>::set(kitty_id + One::one()); // kitty_id+1

			// 触发事件
			Self::deposit_event(Event::KittyCreated(sender.clone(), kitty_id));

			// 返回OK
			Ok(().into())
        }
		
	}
}
