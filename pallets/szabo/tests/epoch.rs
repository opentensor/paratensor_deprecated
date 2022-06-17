use crate::{mock::*};
use pallet_szabo::{Error};
use frame_system::{Config};
use frame_support::assert_ok;
use frame_support::sp_runtime::DispatchError;

mod mock;
mod helpers;

#[test]
fn test_nill_epoch() {
	new_test_ext().execute_with(|| {
		SzaboModule::epoch(0,0);
        SzaboModule::epoch(1,1000);
	});
}

#[test]
fn test_random_1graph() {
	new_test_ext().execute_with(|| {
		helpers::create_random_subgraph(0, 2);
        SzaboModule::epoch(0, 100);
        helpers::print_network_state( 0 );
	});
}

