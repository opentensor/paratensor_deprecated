use frame_support::{assert_ok};
use frame_system::{Config};
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;
use pallet_paratensor::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};

/***********************************************************
	staking::add_stake() tests
************************************************************/

#[test]
fn test_add_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let hotkey = 0;
		let ammount_staked = 5000;
        let call = Call::ParatensorModule(ParatensorCall::add_stake{hotkey, ammount_staked});
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}
#[test]
fn test_add_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 533453;
		let coldkey_account_id = 55453;
        let netuid : u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Register neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Give it some $$$ in his coldkey balance
		ParatensorModule::add_balance_to_coldkey_account( &coldkey_account_id, 10000 );

		// Check we have zero staked before transfer
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_account_id ), 0);

		// Also total stake should be zero
		assert_eq!(ParatensorModule::get_total_stake(), 0);

		// Transfer to hotkey account, and check if the result is ok
		assert_ok!(ParatensorModule::add_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, 10000));

		// Check if stake has increased
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_account_id), 10000);

		// Check if balance has  decreased
		assert_eq!(ParatensorModule::get_coldkey_balance(&coldkey_account_id), 0);

		// Check if total stake has increased accordingly.
		assert_eq!(ParatensorModule::get_total_stake(), 10000);

        // Check if stake has added for each subnetwork that hotkey is regostered on
        assert_eq!(ParatensorModule::get_hotkey_stake_for_subnet(netuid, &hotkey_account_id), 10000);
	});
}

#[test]
fn test_dividends_with_run_to_block() {
	new_test_ext().execute_with(|| {
		let neuron_src_hotkey_id = 1;
		let neuron_dest_hotkey_id = 2;
		let coldkey_account_id = 667;
		let netuid: u16 = 1;

		let initial_stake:u64 = 5000;

		//add network
		add_network(netuid, 13, 0);

		// Register neuron, this will set a self weight
		ParatensorModule::set_max_registrations_per_block( 3 );
		ParatensorModule::set_max_allowed_uids(1, 5);
		
		register_ok_neuron( netuid, 0, coldkey_account_id, 2112321);
		register_ok_neuron(netuid, neuron_src_hotkey_id, coldkey_account_id, 192213123);
		register_ok_neuron(netuid, neuron_dest_hotkey_id, coldkey_account_id, 12323);

		// Add some stake to the hotkey account, so we can test for emission before the transfer takes place
		ParatensorModule::add_stake_to_neuron_hotkey_account(&neuron_src_hotkey_id, initial_stake);

		// Check if the initial stake has arrived
		assert_eq!( ParatensorModule::get_stake_of_neuron_hotkey_account(&neuron_src_hotkey_id), initial_stake );

		// Check if all three neurons are registered
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 3 );

		// Run a couple of blocks to check if emission works
		run_to_block( 2 );

		// Check if the stake is equal to the inital stake + transfer
		assert_eq!(ParatensorModule::get_stake_of_neuron_hotkey_account(&neuron_src_hotkey_id), initial_stake);

		// Check if the stake is equal to the inital stake + transfer
		assert_eq!(ParatensorModule::get_stake_of_neuron_hotkey_account(&neuron_dest_hotkey_id), 0);
    });
}

#[test]
fn test_add_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id = 654; // bogus
		let amount = 20000 ; // Not used

		let result = ParatensorModule::add_stake(<<Test as Config>::Origin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}

#[test]
fn test_add_stake_not_registered_key_pair() { //it must pass since we are not checking if the hotkey is registered for DAO purposes
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 435445; // Not active id
		let hotkey_account_id = 54544;
		let amount = 1337;

		// Put the balance on the account
		ParatensorModule::add_balance_to_coldkey_account(&coldkey_account_id, 1800);
		
		assert_ok!(ParatensorModule::add_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, amount));
	});
}

#[test]
fn test_add_stake_err_neuron_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce : u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		// Give it some $$$ in his coldkey balance
		ParatensorModule::add_balance_to_coldkey_account( &other_cold_key, 100000 );

		// Perform the request which is signed by a different cold key
		let result = ParatensorModule::add_stake(<<Test as Config>::Origin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_add_stake_err_not_enough_belance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 544;
		let hotkey_id = 54544;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Lets try to stake with 0 balance in cold key account
		assert_eq!(ParatensorModule::get_coldkey_balance(&coldkey_id), 0);
		let result = ParatensorModule::add_stake(<<Test as Config>::Origin>::signed(coldkey_id), hotkey_id, 60000);

		assert_eq!(result, Err(Error::<Test>::NotEnoughBalanceToStake.into()));
	});
}

// /***********************************************************
// 	staking::remove_stake() tests
// ************************************************************/

#[test]
fn test_remove_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
        let hotkey = 0;
		let ammount_unstaked = 5000;

		let call = Call::ParatensorModule(ParatensorCall::remove_stake{hotkey, ammount_unstaked});

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_remove_stake_ok_no_emission() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 4343;
		let hotkey_account_id = 4968585;
		let amount = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		// Let's spin up a neuron
		register_ok_neuron( netuid, hotkey_account_id, coldkey_account_id, start_nonce);

		// Some basic assertions
		assert_eq!(ParatensorModule::get_total_stake(), 0);
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(ParatensorModule::get_coldkey_balance(&coldkey_account_id), 0);

		// Give the neuron some stake to remove
		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_account_id, amount);

		// Do the magic
		assert_ok!(ParatensorModule::remove_stake(<<Test as Config>::Origin>::signed(coldkey_account_id), hotkey_account_id, amount));

		assert_eq!(ParatensorModule::get_coldkey_balance(&coldkey_account_id), amount as u128);
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_account_id), 0);
		assert_eq!(ParatensorModule::get_total_stake(), 0);
		assert_eq!(ParatensorModule::get_hotkey_stake_for_subnet(netuid, &hotkey_account_id), 0);
	});
}

#[test]
fn test_remove_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let hotkey_account_id : u64 = 4968585;
		let amount = 10000; // Amount to be removed

		let result = ParatensorModule::remove_stake(<<Test as Config>::Origin>::none(), hotkey_account_id, amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}

#[test]
fn test_remove_stake_err_hotkey_does_not_belong_to_coldkey() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 544;
		let hotkey_id = 54544;
		let other_cold_key = 99498;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Perform the request which is signed by a different cold key
		let result = ParatensorModule::remove_stake(<<Test as Config>::Origin>::signed(other_cold_key), hotkey_id, 1000);
		assert_eq!(result, Err(Error::<Test>::NonAssociatedColdKey.into()));
	});
}

#[test]
fn test_remove_stake_no_enough_stake() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 544;
		let hotkey_id = 54544;
		let amount = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_id), 0);

		let result = ParatensorModule::remove_stake(<<Test as Config>::Origin>::signed(coldkey_id), hotkey_id, amount);
		assert_eq!(result, Err(Error::<Test>::NotEnoughStaketoWithdraw.into()));
	});
}

/***********************************************************
	staking::get_coldkey_balance() tests
************************************************************/
#[test]
fn test_get_coldkey_balance_no_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 5454; // arbitrary
		let result = ParatensorModule::get_coldkey_balance(&coldkey_account_id);

		// Arbitrary account should have 0 balance
		assert_eq!(result, 0);

	});
}

#[test]
fn test_get_coldkey_balance_with_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 5454; // arbitrary
		let amount = 1337;

		// Put the balance on the account
		ParatensorModule::add_balance_to_coldkey_account(&coldkey_account_id, amount);

		let result = ParatensorModule::get_coldkey_balance(&coldkey_account_id);

		// Arbitrary account should have 0 balance
		assert_eq!(result, amount);

	});
}

// /***********************************************************
// 	staking::add_stake_to_hotkey_account() tests
// ************************************************************/
#[test]
fn test_add_stake_to_hotkey_account_ok() {
	new_test_ext().execute_with(|| {
		let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// There is not stake in the system at first, so result should be 0;
		assert_eq!(ParatensorModule::get_total_stake(), 0);

		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_id, amount);

		// The stake that is now in the account, should equal the amount
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_id), amount);

		// The total stake should have been increased by the amount -> 0 + amount = amount
		assert_eq!(ParatensorModule::get_total_stake(), amount);
	});
}

/************************************************************
	staking::remove_stake_from_hotkey_account() tests
************************************************************/
#[test]
fn test_remove_stake_from_hotkey_account() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		// Add some stake that can be removed
		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_id, amount);

		// Prelimiary checks
		assert_eq!(ParatensorModule::get_total_stake(), amount);
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_id), amount);

		// Remove stake
		ParatensorModule::remove_stake_from_hotkey_account(&hotkey_id, amount);

		// The stake on the hotkey account should be 0
		assert_eq!(ParatensorModule::get_stake_for_hotkey(&hotkey_id), 0);

		// The total amount of stake should be 0
		assert_eq!(ParatensorModule::get_total_stake(), 0);
	});
}

#[test]
fn test_remove_stake_from_hotkey_account_registered_in_various_networks() {
	new_test_ext().execute_with(|| {
		let hotkey_id = 5445;
		let coldkey_id = 5443433;
		let amount: u64 = 10000;
        let netuid: u16 = 1;
		let netuid_ex = 2;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;
		//
		add_network(netuid, tempo, 0);
		add_network(netuid_ex, tempo, 0);
		//
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		register_ok_neuron( netuid_ex, hotkey_id, coldkey_id, 48141209);
		
		//let neuron_uid = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_id);
		let neuron_uid ;
        match ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_id) {
            Ok(k) => neuron_uid = k,
            Err(e) => panic!("Error: {:?}", e),
        } 
		//let neuron_uid_ex = ParatensorModule::get_neuron_for_net_and_hotkey(netuid_ex, &hotkey_id);
		let neuron_uid_ex ;
        match ParatensorModule::get_neuron_for_net_and_hotkey(netuid_ex, &hotkey_id) {
            Ok(k) => neuron_uid_ex = k,
            Err(e) => panic!("Error: {:?}", e),
        } 
		//Add some stake that can be removed
		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_id, amount);

		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid, neuron_uid), amount);
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid_ex, neuron_uid_ex), amount);

		// Remove stake
		ParatensorModule::remove_stake_from_hotkey_account(&hotkey_id, amount);
		//
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid, neuron_uid), 0);
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid_ex, neuron_uid_ex), 0);
	});
}


// /************************************************************
// 	staking::increase_total_stake() tests
// ************************************************************/
#[test]
fn test_increase_total_stake_ok() {
	new_test_ext().execute_with(|| {
        let increment = 10000;

        assert_eq!(ParatensorModule::get_total_stake(), 0);
	    ParatensorModule::increase_total_stake(increment);
		assert_eq!(ParatensorModule::get_total_stake(), increment);
	});
}

#[test]
#[should_panic]
fn test_increase_total_stake_panic_overflow() {
	new_test_ext().execute_with(|| {
        let initial_total_stake = u64::MAX;
		let increment : u64 = 1;

		// Setup initial total stake
		ParatensorModule::increase_total_stake(initial_total_stake);
		ParatensorModule::increase_total_stake(increment); // Should trigger panic
	});
}

// /************************************************************
// 	staking::decrease_total_stake() tests
// ************************************************************/
#[test]
fn test_decrease_total_stake_ok() {
	new_test_ext().execute_with(|| {
        let initial_total_stake = 10000;
		let decrement = 5000;

		ParatensorModule::increase_total_stake(initial_total_stake);
		ParatensorModule::decrease_total_stake(decrement);

		// The total stake remaining should be the difference between the initial stake and the decrement
		assert_eq!(ParatensorModule::get_total_stake(), initial_total_stake - decrement);
	});
}

#[test]
#[should_panic]
fn test_decrease_total_stake_panic_underflow() {
	new_test_ext().execute_with(|| {
        let initial_total_stake = 10000;
		let decrement = 20000;

		ParatensorModule::increase_total_stake(initial_total_stake);
		ParatensorModule::decrease_total_stake(decrement); // Should trigger panic
	});
}

// /************************************************************
// 	staking::add_balance_to_coldkey_account() tests
// ************************************************************/
#[test]
fn test_add_balance_to_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 4444322;
		let amount = 50000;

		ParatensorModule::add_balance_to_coldkey_account(&coldkey_id, amount);
		assert_eq!(ParatensorModule::get_coldkey_balance(&coldkey_id), amount);
	});
}

// /***********************************************************
// 	staking::remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_remove_balance_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Put some $$ on the bank
		ParatensorModule::add_balance_to_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(ParatensorModule::get_coldkey_balance(&coldkey_account_id), ammount);

		// Should be able to withdraw without hassle
		let result = ParatensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(result, true);
	});
}

#[test]
fn test_remove_balance_from_coldkey_account_failed() {
	new_test_ext().execute_with(|| {
		let coldkey_account_id = 434324; // Random
		let ammount = 10000; // Arbitrary

		// Try to remove stake from the coldkey account. This should fail,
		// as there is no balance, nor does the account exist
		let result = ParatensorModule::remove_balance_from_coldkey_account(&coldkey_account_id, ammount);
		assert_eq!(result, false);
	});
}

///************************************************************
// 	staking::hotkey_belongs_to_coldkey() tests
// ************************************************************/
#[test]
fn test_hotkey_belongs_to_coldkey_ok() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4434334;
		let coldkey_id = 34333;
        let netuid: u16 = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		assert_eq!(ParatensorModule::get_coldkey_for_hotkey(&hotkey_id), coldkey_id);
	});
}
// /************************************************************
// 	staking::can_remove_balance_from_coldkey_account() tests
// ************************************************************/
#[test]
fn test_can_remove_balane_from_coldkey_account_ok() {
	new_test_ext().execute_with(|| {
        let coldkey_id = 87987984;
		let initial_amount = 10000;
		let remove_amount = 5000;

		ParatensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
		assert_eq!(ParatensorModule::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount), true);
	});
}

#[test]
fn test_can_remove_balance_from_coldkey_account_err_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let coldkey_id = 87987984;
		let initial_amount = 10000;
		let remove_amount = 20000;

		ParatensorModule::add_balance_to_coldkey_account(&coldkey_id, initial_amount);
		assert_eq!(ParatensorModule::can_remove_balance_from_coldkey_account(&coldkey_id, remove_amount), false);
	});
}
/************************************************************
	staking::has_enough_stake() tests
************************************************************/
#[test]
fn test_has_enough_stake_yes() {
	new_test_ext().execute_with(|| {
        let hotkey_id = 4334;
		let coldkey_id = 87989;
		let intial_amount = 10000;
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);

		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_id, intial_amount);
        ParatensorModule::get_stake_for_hotkey(&hotkey_id);
        //
		assert_eq!(ParatensorModule::has_enough_stake(&hotkey_id, 5000), true);
	});
}

#[test]
fn test_has_enough_stake_no() {
	new_test_ext().execute_with(|| {
		let hotkey_id = 4334;
		let coldkey_id = 87989;
		let intial_amount = 0;
        let netuid = 1;
		let tempo: u16 = 13;
		let start_nonce: u64 = 0;

		//add network
		add_network(netuid, tempo, 0);
		
		register_ok_neuron( netuid, hotkey_id, coldkey_id, start_nonce);
		ParatensorModule::add_stake_to_neuron_hotkey_account(&hotkey_id, intial_amount);
		assert_eq!(ParatensorModule::has_enough_stake(&hotkey_id, 5000), false);

	});
}

