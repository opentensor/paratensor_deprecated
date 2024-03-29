use pallet_paratensor::{Error};
use frame_support::{assert_ok};
use frame_system::Config;
//use mock::*;
use crate::{mock::*};
use frame_support::sp_runtime::DispatchError;
use frame_support::dispatch::{GetDispatchInfo, DispatchInfo};
use frame_support::weights::{DispatchClass, Pays};

mod mock;

/********************************************
	subscribing::subscribe() tests
*********************************************/
#[test]
fn test_registration_subscribe_ok_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 0;
		let nonce: u64 = 0;
        let netuid: u16 = 1;
		let work: Vec<u8> = vec![0;32];
		let hotkey: u64 = 0;
		let coldkey: u64 = 0;
        let call = Call::ParatensorModule(ParatensorCall::register{netuid, block_number, nonce, work, hotkey, coldkey });
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_registration_difficulty() {
	new_test_ext().execute_with(|| {
		assert_eq!( ParatensorModule::get_difficulty(1).as_u64(), 10000 );
	});

}

#[test]
fn test_registration_repeat_work() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 0;
		let netuid: u16 = 1;
		let hotkey_account_id_1 = 1;
		let hotkey_account_id_2 = 2;
		let coldkey_account_id = 667; // Neighbour of the beast, har har
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id_1), netuid, block_number, nonce, work.clone(), hotkey_account_id_1, coldkey_account_id));
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id_2), netuid, block_number, nonce, work.clone(), hotkey_account_id_2, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::WorkRepeated.into()) );
	});
}

#[test]
fn test_registration_ok() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 0;
		let netuid: u16 = 1;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 129123813);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id));

		// Check if neuron has added to the specified network(netuid)
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 1);

		//check if hotkey is added to the Hotkeys
		assert_eq!(ParatensorModule::get_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		//check if coldkey is added to coldkeys
		assert_eq!(ParatensorModule::get_hotkey_for_coldkey(&coldkey_account_id), hotkey_account_id);

		// Check the list of neworks that uid has registered 
		let subs = ParatensorModule::get_subnets_for_hotkey(hotkey_account_id);
		assert_eq!(subs.contains(&netuid), true);

		// Check if the neuron has added to the Keys
		let neuron_uid: u16 = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id);
		assert_eq!(ParatensorModule::get_hotkey_for_net_and_neuron(netuid, neuron_uid), hotkey_account_id);

		// Check if neuron has added to Uids
        assert_eq!(ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id), neuron_uid);

		// Check if the balance of this hotkey account for this subnetwork == 0
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid, neuron_uid), 0);
	});
}

#[test]
fn test_registration_too_many_registrations_per_block() {
	new_test_ext().execute_with(|| {
		
		let netuid: u16 = 1;
		ParatensorModule::set_max_registratations_per_block( 10 );

		let block_number: u64 = 0;
		let (nonce0, work0): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 3942084);
		let (nonce1, work1): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 11231312312);
		let (nonce2, work2): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 212312414);
		let (nonce3, work3): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 21813123);
		let (nonce4, work4): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 148141209);
		let (nonce5, work5): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 1245235534);
		let (nonce6, work6): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 256234);
		let (nonce7, work7): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 6923424);
		let (nonce8, work8): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 124242);
		let (nonce9, work9): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 153453);
		let (nonce10, work10): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 345923888);
		assert_eq!( ParatensorModule::get_difficulty_as_u64(netuid), 10000 );

		// Subscribe and check extrinsic output
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(0), netuid, block_number, nonce0, work0, 0, 0));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(1), netuid, block_number, nonce1, work1, 1, 1));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(2), netuid, block_number, nonce2, work2, 2, 2));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(3), netuid, block_number, nonce3, work3, 3, 3));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(4), netuid, block_number, nonce4, work4, 4, 4));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(5), netuid, block_number, nonce5, work5, 5, 5));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(6), netuid, block_number, nonce6, work6, 6, 6));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(7), netuid, block_number, nonce7, work7, 7, 7));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(8), netuid, block_number, nonce8, work8, 8, 8));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(9), netuid, block_number, nonce9, work9, 9, 9));
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(10), netuid, block_number, nonce10, work10, 10, 10);
		assert_eq!( result, Err(Error::<Test>::ToManyRegistrationsThisBlock.into()) );
	});
}

#[test]
fn test_registration_defaults() {
	new_test_ext().execute_with(|| { /*TO DO: step_block calls on_initialize in the hooks that is not fully implemented yet */
		/*let netuid: u16 = 1;
		//
		assert_eq!( ParatensorModule::get_difficulty_as_u64(netuid), 10000 );
		assert_eq!( ParatensorModule::get_target_registrations_per_interval(), 2 );
		assert_eq!( ParatensorModule::get_adjustment_interval(), 100 );
		assert_eq!( ParatensorModule::get_max_registratations_per_block(), 2 );
		step_block ( 1 );
		assert_eq!( ParatensorModule::get_difficulty_as_u64(netuid), 10000 );
		assert_eq!( ParatensorModule::get_target_registrations_per_interval(), 2 );
		assert_eq!( ParatensorModule::get_adjustment_interval(), 100 );
		assert_eq!( ParatensorModule::get_max_registratations_per_block(), 2 );
		Subtensor::set_adjustment_interval( 2 );
		Subtensor::set_target_registrations_per_interval( 2 );
		Subtensor::set_difficulty_from_u64( 2 );
		Subtensor::set_max_registratations_per_block( 2 );
		assert_eq!( Subtensor::get_difficulty_as_u64(), 2 );
		assert_eq!( Subtensor::get_target_registrations_per_interval(), 2 );
		assert_eq!( Subtensor::get_adjustment_interval(), 2 );
		assert_eq!( Subtensor::get_max_registratations_per_block(), 2 ); */
	});
}

#[test]
fn test_registration_difficulty_adjustment() {
	/*TO DO */
}

#[test]
fn test_registration_immunity_period() {
	/* TO DO */
}

#[test]
fn test_registration_already_active_hotkey() {
	new_test_ext().execute_with(|| {

		let block_number: u64 = 0;
		let netuid: u16 = 1;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;

		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id));

		let block_number: u64 = 0;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::AlreadyRegistered.into()) );
	});
}

#[test]
fn test_registration_invalid_seal() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 0;
		let netuid:u16 =1;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, 1, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::InvalidSeal.into()) );
	});
}

#[test]
fn test_registration_invalid_block_number() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 1;
		let netuid: u16 =1;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number(netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::InvalidWorkBlock.into()) );
	});
}

#[test]
fn test_registration_invalid_difficulty() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 0;
		let netuid: u16 = 1;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		ParatensorModule::set_difficulty_from_u64( netuid, 18_446_744_073_709_551_615u64 );
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::InvalidDifficulty.into()) );
	});
}

#[test]
fn test_registration_failed_no_signature() {
	new_test_ext().execute_with(|| {

		let block_number: u64 = 1;
		let netuid: u16 = 1;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		// Subscribe and check extrinsic output
		let result = ParatensorModule::register(<<Test as Config>::Origin>::none(), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!(result, Err(DispatchError::BadOrigin.into()));
	});
}

#[test]
fn test_registration_get_next_uid() {
	new_test_ext().execute_with(|| {
        assert_eq!(ParatensorModule::get_next_uid(), 0); // We start with id 0
		assert_eq!(ParatensorModule::get_next_uid(), 1); // One up
		assert_eq!(ParatensorModule::get_next_uid(), 2) // One more
	});
}

#[test]
fn test_registration_get_uid_to_prune() {
	new_test_ext().execute_with(|| {
		ParatensorModule::set_prunning_score(1,1,100);
		ParatensorModule::set_prunning_score(1,2,110);
		ParatensorModule::set_prunning_score(1,3,120);
		//
		assert_eq!(ParatensorModule::get_neuron_to_prune(1), 1);
	});
}

#[test]
fn test_registration_pruning() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let block_number: u64 = 0;
		let (nonce0, work0): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 3942084);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce0, work0, hotkey_account_id, coldkey_account_id));
		let neuron_uid = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id);
		ParatensorModule::set_prunning_score(netuid, neuron_uid, 2);
		//
		let (nonce1, work1): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 11231312312);
		let hotkey_account_id1 = 2;
		let coldkey_account_id1 = 668;
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id1), netuid, block_number, nonce1, work1, hotkey_account_id1, coldkey_account_id1));
		let neuron_uid1 = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &hotkey_account_id1);
		ParatensorModule::set_prunning_score(netuid, neuron_uid1, 3);
		//
		let (nonce2, work2): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 212312414);
		let hotkey_account_id2 = 3;
		let coldkey_account_id2 = 669;
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id2), netuid, block_number, nonce2, work2, hotkey_account_id2, coldkey_account_id2));
		//
		let subs = ParatensorModule::get_subnets_for_hotkey(hotkey_account_id);
		assert_eq!(subs.contains(&netuid), false);
		//
		assert_eq!(ParatensorModule::if_emission_is_set_for_neuron(netuid, neuron_uid), false);
		assert_eq!(ParatensorModule::if_weights_is_set_for_neuron(netuid, neuron_uid), false);
		assert_eq!(ParatensorModule::if_rank_is_set_for_neuron(netuid, neuron_uid), false);
		assert_eq!(ParatensorModule::if_trust_is_set_for_neuron(netuid, neuron_uid), false);
		assert_eq!(ParatensorModule::if_incentive_is_set_for_neuron(netuid, neuron_uid), false);
		assert_eq!(ParatensorModule::if_consensus_is_set_for_neuron(netuid, neuron_uid), false);
		assert_eq!(ParatensorModule::if_dividend_is_set_for_neuron(netuid, neuron_uid), false);
	});
}