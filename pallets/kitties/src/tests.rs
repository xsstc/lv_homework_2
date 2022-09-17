use super::*; // 引入Kitties模块
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// 小结：用assert_ok去测试执行方法是否OK，用assert_eq去对比值是否相等，用assert_noop去测试异常情况。
//      先对每个方法按正常的情况测试一遍，再根据Error里定义的内容，去覆盖所有异常的场景。

#[test]
// 测试创建Kitty，应该操作成功
fn create_works() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 检查下一个kitty_id，应该为1
		assert_eq!(NextKittyId::<Test>::get(), 1);
		// 获取index=0的Kitty，账户应该为1
		assert_eq!(KittyOwner::<Test>::get(0), Some(1));
		// 触发创建事件
		// System::assert_has_event(Event::<Test>::KittyCreated(1, 0));
	});
}

#[test]
// 测试创建Kitty，当kitty_id溢出时，应该操作失败
fn create_failed_when_kittyid_overflow() {
	new_test_ext().execute_with(|| {
		// 设置NextKittyId为u32的最大值，再+1就会溢出
		NextKittyId::<Test>::put(u32::max_value()); // u32::max_value() 获取u32最大值
		let account_id: u64 = 1;
		// 创建一个Kitty，此时NextKittyId+1应该溢出
		// assert_noop!(KittiesModule::create(Origin::signed(account_id)), Error::<Test>::KittyIdOverflow);
	});
}

#[test]
// 测试创建Kitty，当没有足够的余额质押时，应该操作失败
fn create_failed_when_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 3;
		// 用账户3（余额9_000，见mock.rs定义）去创建一个Kitty，由于创建Kitty需要质押10_000个token，所以应该提示余额不足
		assert_noop!(KittiesModule::create(Origin::signed(account_id)), Error::<Test>::NotEnoughBalance);

	});
}

#[test]
// 测试转移Kitty，应该操作成功
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 用账户1去转移index=0的Kitty给账户2
		assert_ok!(KittiesModule::transfer(Origin::signed(account_id), 0, 2));
		// 检查index=0的Kitty的归属，应该属于账户2
		assert_eq!(KittyOwner::<Test>::get(0), Some(2));
		// 触发转移事件
		// System::assert_has_event(Event::<Test>::KittyTransferred(1, 2, 0));
	});
}

#[test]
// 测试转移Kitty，当前账户不是Owner时，应该操作失败
fn transfer_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 用账户2去转移index=0的Kitty给账户3，应该提示账户2不是index=0的Kitty的Owner
		assert_noop!(KittiesModule::transfer(Origin::signed(2), 0, 3), Error::<Test>::NotOwner);
	});
}

#[test]
// 测试转移Kitty，当转移Kitty给新Owner，且Owner没有足够的余额来质押时，应该操作失败
fn transfer_failed_when_new_owner_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let account_id: u64 = 1;
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(account_id)));
		// 用账户1去转移index=0的Kitty给账户3，由于账户3的余额（9_000）不够质押（10_000），应该提示账户3余额不够质押
		assert_noop!(KittiesModule::transfer(Origin::signed(account_id), 0, 3), Error::<Test>::NotEnoughBalance);
	});
}

#[test]
// 测试繁殖Kitty，应该操作成功
fn breed_works() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		// 用账户2去创建index=1的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(2u64)));
		// 用账户1、index=0的Kitty、index=1的Kitty去繁殖新Kitty
		assert_ok!(KittiesModule::breed(Origin::signed(1u64), 0, 1));
		// 此时NextKittyId应该为3个
		assert_eq!(NextKittyId::<Test>::get(), 3);
		// 触发创建事件
		// System::assert_has_event(Event::<Test>::KittyCreated(1, 2));
	});
}


#[test]
// 测试繁殖Kitty，当KittyIndex不存在时，应该操作失败
fn breed_failed_when_invalid_kitty_id() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		// 用账户2去创建index=1的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(2u64)));
		// 用账户1、index=0的Kitty、index=3的Kitty去繁殖新Kitty，由于index=3的Kitty不存在，应该提示无效的KittyId
		assert_noop!(KittiesModule::breed(Origin::signed(1u64), 0, 3), Error::<Test>::InvalidKittyId);
	});
}

#[test]
// 测试繁殖Kitty，当繁殖父母为同一只Kitty时，应该操作失败
fn breed_failed_when_same_parent() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		// 用账户2去创建index=1的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(2u64)));
		// 用账户1、两只index=0的Kitty去繁殖，应该提示父母不能是同一只
		assert_noop!(KittiesModule::breed(Origin::signed(1u64), 0, 0), Error::<Test>::SameKittyId);
	});
}

#[test]
// 测试繁殖Kitty，当没有足够的余额用于质押时，应该操作失败
fn breed_failed_when_not_enough_balance_for_staking() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		// 用账户2去创建index=1的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(2u64)));
		// 用账户3、index=0的Kitty、index=1的Kitty去繁殖，应该提示账户3余额不够质押
		assert_noop!(KittiesModule::breed(Origin::signed(3u64), 0, 1), Error::<Test>::NotEnoughBalance);
	});
}

#[test]
// 测试出售Kitty，将Kitty报价并上架出售，应该操作成功
fn sell_works() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		let price: u128 = 1_500;
		// 用账户1给index=0的Kitty报价1_500并上架出售
		assert_ok!(KittiesModule::sell(Origin::signed(1u64), 0, Some(price)));
		// 检查index=0的Kitty的价格，应该为1_500
		assert_eq!(KittiesShop::<Test>::get(0), Some(price));
		// 触发出售事件
		// System::assert_has_event(Event::<Test>::KittyInSell(1, 0, Some(price)));
	});
}

#[test]
// 测试出售Kitty，当前账户不是Kitty的Owner时，应该操作失败
fn sell_failed_when_not_owner() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		let price: u128 = 1_500;
		// 用账户3去出售index=0的Kitty，应该提示不是Kitty的Owner
		assert_noop!(KittiesModule::sell(Origin::signed(3u64), 0, Some(price)), Error::<Test>::NotOwner);
	});
}

#[test]
// 测试购买Kitty，应该操作成功
fn buy_works() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		let price: u128 = 1_500;
		// 用账户1给index=0的Kitty报价并上架出售
		assert_ok!(KittiesModule::sell(Origin::signed(1u64), 0, Some(price)));
		// 用账户2去购买index=0的Kitty
		assert_ok!(KittiesModule::buy(Origin::signed(2u64), 0));
		// 此时index=0的Kitty的Owner应该是账户2
		assert_eq!(KittyOwner::<Test>::get(0), Some(2u64));
		// 触发转移事件
		// System::assert_has_event(Event::<Test>::KittyTransferred(1, 2, 0));
	});
}

#[test]
// 测试购买Kitty，当买卖双方是同一个账户时，应该操作失败
fn buy_failed_when_buyer_is_owner() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		let price: u128 = 1_500;
		// 用账户1给index=0的Kitty报价并上架出售
		assert_ok!(KittiesModule::sell(Origin::signed(1u64), 0, Some(price)));
		// 用账户1去购买index=0的Kitty，应该提示不能买自己的Kitty
		assert_noop!(KittiesModule::buy(Origin::signed(1u64), 0), Error::<Test>::NoBuySelf);
	});
}

#[test]
// 测试购买Kitty，当Kitty的报价为None（非卖品）时，应该操作失败
fn buy_failed_when_not_for_sale() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		// 用账户1给index=0的Kitty报空价并上架出售
		assert_ok!(KittiesModule::sell(Origin::signed(1u64), 0, None));
		// 用账户2去购买index=0的Kitty，应该提示不出售（非卖品）
		assert_noop!(KittiesModule::buy(Origin::signed(2u64), 0), Error::<Test>::NotForSale);
	});
}

#[test]
// 测试购买Kitty，当没有足够的余额来购买和质押Kitty时，应该操作失败
fn buy_failed_when_not_enough_balance() {
	new_test_ext().execute_with(|| {
		// 用账户1去创建index=0的Kitty
		assert_ok!(KittiesModule::create(Origin::signed(1u64)));
		// 用账户1给index=0的Kitty报价并上架出售
		assert_ok!(KittiesModule::sell(Origin::signed(1u64), 0, Some(1_500)));
		// 用账户3去购买index=0的Kitty，应该提示余额不足
		assert_noop!(KittiesModule::buy(Origin::signed(3u64), 0), Error::<Test>::NotEnoughBalance);
	});
}
