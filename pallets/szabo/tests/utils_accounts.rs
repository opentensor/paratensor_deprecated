use crate::{mock::*};

mod mock;
mod helpers;

#[test]
fn test_initial_total_staked() {
	new_test_ext().execute_with(|| {
        assert_eq!( SzaboModule::get_total_stake(), 0 );
    });
}

#[test]
fn test_initial_total_issuance() {
	new_test_ext().execute_with(|| {
        assert_eq!( SzaboModule::get_total_issuance(), 0 );
    });
}

#[test]
fn test_global_n() {
	new_test_ext().execute_with(|| {
        assert_eq!( SzaboModule::get_global_n(), 0 );
        SzaboModule::increment_global_n();
        assert_eq!( SzaboModule::get_global_n(), 1 );
        SzaboModule::increment_global_n();
        assert_eq!( SzaboModule::get_global_n(), 2 );
        SzaboModule::decrement_global_n();
        assert_eq!( SzaboModule::get_global_n(), 1 );
        SzaboModule::decrement_global_n();
        assert_eq!( SzaboModule::get_global_n(), 0 );
        SzaboModule::decrement_global_n();
        assert_eq!( SzaboModule::get_global_n(), 0 );

    });
}

#[test]
fn test_subnetwork_n() {
	new_test_ext().execute_with(|| {
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 0 );
        SzaboModule::increment_subnetwork_n( 0 );
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 1 );
        SzaboModule::increment_subnetwork_n( 0 );
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 2 );
        SzaboModule::decrement_subnetwork_n( 0 );
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 1 );
        SzaboModule::decrement_subnetwork_n( 0 );
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 0 );
        SzaboModule::decrement_subnetwork_n( 0 );
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 0 );
        SzaboModule::increment_subnetwork_n( 0 );
        assert_eq!( SzaboModule::get_subnetwork_n( 0 ), 1 );
        SzaboModule::increment_subnetwork_n( 1 );
        assert_eq!( SzaboModule::get_subnetwork_n( 1 ), 1 );
        SzaboModule::increment_subnetwork_n( 1 );
        assert_eq!( SzaboModule::get_subnetwork_n( 1 ), 2 );
    });
}

#[test]
fn test_add_global_account() {
	new_test_ext().execute_with(|| {
        SzaboModule::add_global_account( &0, &0 );
        assert_eq!( SzaboModule::get_global_n(), 1 );
        assert!( SzaboModule::is_hotkey_globally_active( &0 ) );
        assert_eq!( SzaboModule::get_global_n(), 1 );
        SzaboModule::remove_global_account( &0 );
        assert_eq!( SzaboModule::get_global_n(), 0 );
        SzaboModule::add_global_account( &0, &0 );
        SzaboModule::add_global_account( &0, &0 );
        assert_eq!( SzaboModule::get_global_n(), 1 );
        assert!( SzaboModule::is_hotkey_globally_active( &0 ) );
    });
}

#[test]
fn test_add_subnetwork_account() {
	new_test_ext().execute_with(|| {
        SzaboModule::add_subnetwork_account( 0, 0, &0 );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 0, &0 ) );
        SzaboModule::remove_subnetwork_account( 0, 0 );
        SzaboModule::add_subnetwork_account( 0, 0, &0 );
        SzaboModule::add_subnetwork_account( 0, 0, &0 );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 0, &0 ) );
        SzaboModule::add_subnetwork_account( 0, 1, &0 );
        SzaboModule::add_subnetwork_account( 0, 2, &1 );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 0, &0 ) );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 0, &1 ) );
        assert!( !SzaboModule::is_hotkey_subnetwork_active( 0, &2 ) );
    });
}
