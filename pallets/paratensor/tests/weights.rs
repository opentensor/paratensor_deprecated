mod mock;
use mock::*;
use pallet_paratensor::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;

/***************************
  pub fn set_weights() tests
*****************************/

// Test the call passes through the paratensor module.
#[test]
fn test_set_weights_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let dests = vec![1, 1];
		let weights = vec![1, 1];
        let netuid: u16 = 1;
		let call = Call::ParatensorModule(ParatensorCall::set_weights{netuid, dests, weights});
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

/// Test ensures that uids -- weights must have the same size.
#[test]
fn test_weights_err_weights_vec_not_equal_size() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id:u64 = 55;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
    	register_ok_neuron(1, hotkey_account_id, 66, 0);
		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5]; // Uneven sizes
		let result = ParatensorModule::set_weights(Origin::signed(hotkey_account_id), 1, weights_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
	});
}

/// Test ensures that uids can have not duplicates
#[test]
fn test_weights_err_has_duplicate_ids() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		register_ok_neuron( 1, 666, 77, 0);
		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 6, 6]; // Contains duplicates
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];
		let result = ParatensorModule::set_weights(Origin::signed(666), 1, weights_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
	});
}

/// Test ensures weights cannot exceed max weight limit.
#[test]
fn test_weights_err_max_weight_limit() { //TO DO SAM: uncomment when we implement run_to_block fn
	new_test_ext().execute_with(|| { 
		// Add network.
		let netuid: u16 = 1;
		let tempo: u16 = 100;
		add_network(netuid, tempo, 0);

		// Set params.
		ParatensorModule::set_max_allowed_uids(netuid, 5);
		ParatensorModule::set_max_weight_limit( netuid, u16::MAX/5 );

		// Add 5 accounts.
		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 0, 0 );
		register_ok_neuron( netuid, 0, 0, 55555 );
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 1 );
		assert!( ParatensorModule::is_hotkey_registered( netuid, &0 ) );
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 1, 1 );
		register_ok_neuron( netuid, 1, 1, 65555 );
		assert!( ParatensorModule::is_hotkey_registered( netuid, &1 ) );
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 2);
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 2, 2 );
		register_ok_neuron( netuid, 2, 2, 75555 );
		assert!( ParatensorModule::is_hotkey_registered( netuid, &2 ) );
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 3 );
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 3, 3 );
		register_ok_neuron( netuid, 3, 3, 95555 );
		assert!( ParatensorModule::is_hotkey_registered( netuid, &3 ) );
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 4);
		step_block(1);

		println!( "+Registering: net:{:?}, cold:{:?}, hot:{:?}", netuid, 4, 4 );
		register_ok_neuron( netuid, 4, 4, 35555 );
		assert!( ParatensorModule::is_hotkey_registered( netuid, &4 ) );
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 5);
		step_block(1);

		// Non self-weight fails.
		let uids: Vec<u16> = vec![ 1, 2, 3, 4 ]; 
		let values: Vec<u16> = vec![ u16::MAX/4, u16::MAX/4, u16::MAX/54, u16::MAX/4];
		let result = ParatensorModule::set_weights( Origin::signed(0), 1, uids, values );
		assert_eq!(result, Err(Error::<Test>::MaxWeightExceeded.into()));

		// Self-weight is a success.
		let uids: Vec<u16> = vec![ 0 ];  // Self.
		let values: Vec<u16> = vec![ u16::MAX ]; // normalizes to u32::MAX
		assert_ok!(ParatensorModule::set_weights( Origin::signed(0), 1, uids, values ));
	});
}

/// Tests the call requires a valid origin.
#[test]
fn test_no_signature() {
	new_test_ext().execute_with(|| {
		let uids: Vec<u16> = vec![];
		let values: Vec<u16> = vec![];
		let result = ParatensorModule::set_weights(Origin::none(), 1, uids, values);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

/// Tests that weights cannot be set to non registered uids.
#[test]
fn test_set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
		add_network(1, 13, 0);
		let result = ParatensorModule::set_weights(Origin::signed(1), 1, weights_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::NotRegistered.into()));
	});
}

// Tests that set weights fails if you pass invalid uids.
#[test]
fn test_set_weights_err_invalid_uid() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);
		register_ok_neuron( 1, 55, 66, 0);
		let weight_keys : Vec<u16> = vec![9999]; // Does not exist
		let weight_values : Vec<u16> = vec![88]; // random value
		let result = ParatensorModule::set_weights(Origin::signed(55), 1, weight_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));
	});
}

// Tests that set weights fails if you dont pass enough values.
#[test]
fn test_set_weight_not_enough_values() {
	new_test_ext().execute_with(|| {
        
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		add_network(netuid, tempo, 0);		
		register_ok_neuron(1, 1, 2, 100000);
		register_ok_neuron(1, 3, 4, 300000);
		ParatensorModule::set_min_allowed_weights(1, 2);

		// Should fail because we are only setting a single value and its not the self weight.
		let weight_keys : Vec<u16> = vec![1]; // not weight. 
		let weight_values : Vec<u16> = vec![88]; // random value.
		let result = ParatensorModule::set_weights(Origin::signed(1), 1, weight_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::NotSettingEnoughWeights.into()));

		// Shouldnt fail because we setting a single value but it is the self weight.
		let weight_keys : Vec<u16> = vec![0]; // self weight.
		let weight_values : Vec<u16> = vec![88]; // random value.
		assert_ok!( ParatensorModule::set_weights(Origin::signed(1), 1 , weight_keys, weight_values)) ;

		// Should pass because we are setting enough values.
		let weight_keys : Vec<u16> = vec![0, 1]; // self weight. 
		let weight_values : Vec<u16> = vec![10, 10]; // random value.
		ParatensorModule::set_min_allowed_weights(1, 1);
		assert_ok!( ParatensorModule::set_weights(Origin::signed(1), 1,  weight_keys, weight_values)) ;
	});
}