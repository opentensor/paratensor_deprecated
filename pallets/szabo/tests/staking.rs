use crate::{mock::*};
use pallet_szabo::{Error};
use frame_system::{Config};
use frame_support::assert_ok;
use frame_support::sp_runtime::DispatchError;

mod mock;

#[test]
fn test_account_not_active() {
	new_test_ext().execute_with(|| {
		let result = SzaboModule::add_stake(Origin::signed(0), 0, 0);
		assert_eq!( result, Err(Error::<Test>::NotRegistered.into()) );
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_remove_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let result = SzaboModule::remove_stake(<<Test as Config>::Origin>::none(), 0, 0);
		assert_eq!(result, DispatchError::BadOrigin.into());
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_add_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let result = SzaboModule::add_stake(<<Test as Config>::Origin>::none(), 0, 0);
		assert_eq!(result, DispatchError::BadOrigin.into());
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_remove_stake_wrong_coldkey() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		let result = SzaboModule::remove_stake(Origin::signed(1), 0, 0);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_add_stake_wrong_coldkey() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		let result = SzaboModule::add_stake(Origin::signed(1), 0, 0);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_account_is_active() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(0), 0, 0));
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_not_enough_stake() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		let result =  SzaboModule::add_stake(Origin::signed(0), 0, 10) ;
		assert_eq! ( result, Err(Error::<Test>::NotEnoughBalanceToStake.into()) );
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_add_non_zero_stake() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		SzaboModule::add_balance_to_coldkey_account( &0, 100000 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(0), 0, 100000));
		assert_eq!(SzaboModule::get_total_stake(), 100000);
	});
}

#[test]
fn test_add_remove_stake() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		SzaboModule::add_balance_to_coldkey_account( &0, 100000 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(0), 0, 100000));
		assert_eq!(SzaboModule::get_total_stake(), 100000);
		assert_ok!(SzaboModule::remove_stake(Origin::signed(0), 0, 100000));
		assert_eq!(SzaboModule::get_coldkey_balance(&0), 100000);
		assert_eq!(SzaboModule::get_total_stake(), 0);
	});
}

#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_balance_to_coldkey_account(&0, 10000);
		assert_eq!(SzaboModule::can_remove_balance_from_coldkey_account(&0, 10000), true);
	});
}

#[test]
fn test_can_remove_balane_from_coldkey_account_insufficient_balance() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_balance_to_coldkey_account(&0, 10000);
		assert_eq!(SzaboModule::can_remove_balance_from_coldkey_account(&0, 10000 + 1), false);
	});
}

#[test]
fn test_has_enough_stake_yes() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		SzaboModule::add_balance_to_coldkey_account( &0, 100000 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(0), 0, 100000));
		assert_eq!(SzaboModule::has_enough_stake(&0, 100000), true);
		assert_eq!(SzaboModule::get_total_stake(), 100000);
	});
}

#[test]
fn test_does_not_enough_stake_yes() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		SzaboModule::add_balance_to_coldkey_account( &0, 100000 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(0), 0, 100000));
		assert_eq!(SzaboModule::has_enough_stake(&0, 100000 + 1), false);
		assert_eq!(SzaboModule::get_total_stake(), 100000);
	});
}

#[test]
fn test_get_stake_on_hotkey_account() {
	new_test_ext().execute_with(|| {
		SzaboModule::add_account( &0, &0 );
		SzaboModule::add_stake_to_hotkey_account( &0, 100000 );
		assert_eq!(SzaboModule::get_stake_on_hotkey_account( &0 ), 100000);
		assert_eq!(SzaboModule::get_total_stake(), 100000);
	});
}
