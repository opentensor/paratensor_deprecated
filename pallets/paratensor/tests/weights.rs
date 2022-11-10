mod mock;
use mock::*;
use pallet_paratensor::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;

/***************************
  pub fn set_weights() tests
*****************************/

/*TO DO SAM: write a test to add network and check if network exist */
// This does not produce the expected result
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

/**
* This test the situation where user tries to set weights, but the vecs are empty.
*/
#[test]
fn set_weights_ok_no_weights() {
	new_test_ext().execute_with(|| {

		// == Intial values ==
		let hotkey_account_id:u64 = 55; // Arbitrary number
		let initial_stake = 10000;
        let netuid: u16 = 1;

		let weights_keys : Vec<u16> = vec![];
		let weight_values : Vec<u16> = vec![];

		// == Expectations ==
		let expect_stake:u64 = 10000; // The stake for the neuron should remain the same
		let expect_total_stake:u64 = 10000; // The total stake should remain the same

		//add network
		add_network(netuid, 0);
		
		// Let's subscribe a new neuron to the chain
		register_ok_neuron( netuid, hotkey_account_id, 66, 0);

		// Let's give it some stake.
		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_account_id, initial_stake);

        let neuron_uid = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id);

		// Dispatch a signed extrinsic, setting weights.
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 0);
		assert_ok!(ParatensorModule::set_weights(Origin::signed(hotkey_account_id), netuid, weights_keys, weight_values));
		assert_eq!(ParatensorModule::get_weights_for_neuron(netuid, neuron_uid), vec![0]);
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid, neuron_uid), expect_stake);
		assert_eq!(ParatensorModule::get_total_stake(), expect_total_stake);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 0);

	});
}

#[test]
fn test_priority_increments() { //TO DO SAM: uncomment when step_block fn is implemented
	new_test_ext().execute_with(|| {
		/*let hotkey_account_id:u64 = 55; // Arbitrary number
        let netuid: u16 = 1;

		register_ok_neuron(netuid, hotkey_account_id, 66, 0);
        let neuron_uid = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id);

		ParatensorModule::add_stake_to_neuron_hotkey_account( &hotkey_account_id, 2 );
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 0);
		assert_ok!(ParatensorModule::set_weights(Origin::signed(hotkey_account_id), netuid, vec![], vec![]));
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 0);

        step_block (1);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 1);

        step_block (1);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 2);
		assert_ok!(ParatensorModule::set_weights(Origin::signed(hotkey_account_id), netuid, vec![], vec![]));
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 0);

        step_block (1);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 1);
		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_account_id, 32);

        step_block (1);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 6);

        step_block (1);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 11);
		assert_ok!(ParatensorModule::set_weights(Origin::signed(hotkey_account_id), netuid, vec![], vec![]));
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 0);

        step_block (1);
		assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_uid), 5); */
	});
}

#[test]
fn test_weights_err_weights_vec_not_equal_size() {
	new_test_ext().execute_with(|| {
        let hotkey_account_id:u64 = 55;
		let netuid: u16 = 1;
		//add network
		add_network(netuid, 0);

    	register_ok_neuron(1, hotkey_account_id, 66, 0);

		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5]; // Uneven sizes

		let result = ParatensorModule::set_weights(Origin::signed(hotkey_account_id), 1, weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::WeightVecNotEqualSize.into()));
	});
}

#[test]
fn test_weights_err_has_duplicate_ids() {
	new_test_ext().execute_with(|| {
    	
		let netuid: u16 = 1;
		//add network
		add_network(netuid, 0);

		register_ok_neuron( 1, 666, 77, 0);

		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 6, 6]; // Contains duplicates
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8];

		let result = ParatensorModule::set_weights(Origin::signed(666), 1, weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::DuplicateUids.into()));
	});
}

#[test]
fn test_weights_err_max_weight_limit() { //TO DO SAM: uncomment when we implement run_to_block fn
	new_test_ext().execute_with(|| { /* 
		register_ok_neuron(1, 0, 0, 0);
		run_to_block( 2 );
    	register_ok_neuron( 1, 1, 1, 0);
		run_to_block( 3 );
		register_ok_neuron(1,  2, 2, 0);
		run_to_block( 4 );
    	register_ok_neuron( 1, 3, 3, 0);
		run_to_block( 5 );
    	register_ok_neuron( 1, 4, 4, 0);

		ParatensorModule::set_max_weight_limit(1, u16::MAX/5); // Set max to u16::MAX/5

		// Non self weight fails.
		let weights_keys: Vec<u32> = vec![1, 2, 3, 4]; 
		let weight_values: Vec<u32> = vec![1, 1, 1, 1]; // normalizes to u32::MAX/4
		let result = ParatensorModule::set_weights(Origin::signed(0), 1, weights_keys, weight_values);
		assert_eq!(result, Err(Error::<Test>::MaxWeightExceeded.into()));

		// Self weight is a success.
		let weights_keys: Vec<u32> = vec![0]; 
		let weight_values: Vec<u32> = vec![1]; // normalizes to u32::MAX
		assert_ok!(ParatensorModule::set_weights(Origin::signed(0), 1, weights_keys, weight_values)); */
	});
}

#[test]
fn test_no_signature() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<u16> = vec![];
		let weight_values: Vec<u16> = vec![];

		let result = ParatensorModule::set_weights(Origin::none(), 1, weights_keys, weight_values);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

#[test]
fn test_set_weights_err_not_active() {
	new_test_ext().execute_with(|| {
		let weights_keys: Vec<u16> = vec![1, 2, 3, 4, 5, 6];
		let weight_values: Vec<u16> = vec![1, 2, 3, 4, 5, 6];

		let result = ParatensorModule::set_weights(Origin::signed(1), 1, weights_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::NotRegistered.into()));
	});
}

#[test]
fn test_set_weights_err_invalid_uid() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
		//add network
		add_network(netuid, 0);
		
		register_ok_neuron( 1, 55, 66, 0);
		let weight_keys : Vec<u16> = vec![9999]; // Does not exist
		let weight_values : Vec<u16> = vec![88]; // random value

		let result = ParatensorModule::set_weights(Origin::signed(55), 1, weight_keys, weight_values);

		assert_eq!(result, Err(Error::<Test>::InvalidUid.into()));

	});
}

#[test]
fn test_set_weight_not_enough_values() {
	new_test_ext().execute_with(|| {
        
		let netuid: u16 = 1;
		//add network
		add_network(netuid, 0);
		
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