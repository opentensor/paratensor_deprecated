use pallet_paratensor::{Error, AxonMetadataOf};
use frame_support::{assert_ok};
use frame_system::Config;
use crate::{mock::*};
use frame_support::sp_runtime::DispatchError;
use frame_support::dispatch::{GetDispatchInfo, DispatchInfo};
use frame_support::weights::{DispatchClass, Pays};

mod mock;

/********************************************
	subscribing::subscribe() tests
*********************************************/

/// Tests a basic registration dispatch passes.
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
		let tempo: u16 = 13;
		let hotkey_account_id_1 = 1;
		let hotkey_account_id_2 = 2;
		let coldkey_account_id = 667; // Neighbour of the beast, har har
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		
		//add network
		add_network(netuid, tempo, 0);
		
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
		let tempo: u16 = 13;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 129123813);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667; // Neighbour of the beast, har har

		//add network
		add_network(netuid, tempo, 0);
		
		// Subscribe and check extrinsic output
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id));

		// Check if neuron has added to the specified network(netuid)
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 1);

		//check if hotkey is added to the Hotkeys
		assert_eq!(ParatensorModule::get_owning_coldkey_for_hotkey(&hotkey_account_id), coldkey_account_id);

		// Check if the neuron has added to the Keys
		let neuron_uid = ParatensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
		
		assert!(ParatensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).is_ok());
		// Check if neuron has added to Uids
		let neuro_uid = ParatensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
		assert_eq!(neuro_uid, neuron_uid);

		// Check if the balance of this hotkey account for this subnetwork == 0
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid, neuron_uid), 0);
	});
}

#[test]
fn test_registration_too_many_registrations_per_block() {
	new_test_ext().execute_with(|| {
		
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		ParatensorModule::set_max_registrations_per_block( netuid, 10 );
		assert_eq!( ParatensorModule::get_max_registratations_per_block(netuid), 10 );

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

		//add network
		add_network(netuid, tempo, 0);
		
		// Subscribe and check extrinsic output
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(0), netuid, block_number, nonce0, work0, 0, 0));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 1 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(1), netuid, block_number, nonce1, work1, 1, 1));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 2 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(2), netuid, block_number, nonce2, work2, 2, 2));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 3 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(3), netuid, block_number, nonce3, work3, 3, 3));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 4 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(4), netuid, block_number, nonce4, work4, 4, 4));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 5 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(5), netuid, block_number, nonce5, work5, 5, 5));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 6 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(6), netuid, block_number, nonce6, work6, 6, 6));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 7 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(7), netuid, block_number, nonce7, work7, 7, 7));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 8 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(8), netuid, block_number, nonce8, work8, 8, 8));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 9 );
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(9), netuid, block_number, nonce9, work9, 9, 9));
		assert_eq!( ParatensorModule::get_registrations_this_block(netuid), 10 );
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(10), netuid, block_number, nonce10, work10, 10, 10);
		assert_eq!( result, Err(Error::<Test>::TooManyRegistrationsThisBlock.into()) );
	});
}

#[test]
fn test_registration_immunity_period() { //impl this test when epoch impl and calculating pruning score is done
	/* TO DO */
}

#[test]
fn test_registration_already_active_hotkey() {
	new_test_ext().execute_with(|| {

		let block_number: u64 = 0;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;

		//add network
		add_network(netuid, tempo, 0);
		
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
		let tempo: u16 = 13;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, 1, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;

		//add network
		add_network(netuid, tempo, 0);

		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::InvalidSeal.into()) );
	});
}

#[test]
fn test_registration_invalid_block_number() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 1;
		let netuid: u16 =1;
		let tempo: u16 = 13;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number(netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		
		//add network
		add_network(netuid, tempo, 0);
		
		let result = ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce, work, hotkey_account_id, coldkey_account_id);
		assert_eq!( result, Err(Error::<Test>::InvalidWorkBlock.into()) );
	});
}

#[test]
fn test_registration_invalid_difficulty() {
	new_test_ext().execute_with(|| {
		let block_number: u64 = 0;
		let netuid: u16 = 1;
		let tempo: u16 = 13;
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 0);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		//add network
		add_network(netuid, tempo, 0);

		assert_ok!(ParatensorModule::sudo_set_difficulty( <<Test as Config>::Origin>::root(), netuid, 18_446_744_073_709_551_615u64 ));
		
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
fn test_registration_get_uid_to_prune() {
	new_test_ext().execute_with(|| {
		ParatensorModule::set_pruning_score(1,1,100);
		ParatensorModule::set_pruning_score(1,2,110);
		ParatensorModule::set_pruning_score(1,3,120);
		//
		assert_eq!(ParatensorModule::get_neuron_to_prune(1), 1);
	});
}

#[test]
fn test_registration_pruning() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let block_number: u64 = 0;
		let tempo: u16 = 13;
		let (nonce0, work0): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 3942084);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;
		
		//add network
		add_network(netuid, tempo, 0);
		
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce0, work0, hotkey_account_id, coldkey_account_id));
		//
		let neuron_uid = ParatensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id).unwrap();
		ParatensorModule::set_pruning_score(netuid, neuron_uid, 2);
		//
		let (nonce1, work1): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 11231312312);
		let hotkey_account_id1 = 2;
		let coldkey_account_id1 = 668;
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id1), netuid, block_number, nonce1, work1, hotkey_account_id1, coldkey_account_id1));
		//
		let neuron_uid1 = ParatensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id1).unwrap();
		ParatensorModule::set_pruning_score(netuid, neuron_uid1, 3);
		//
		let (nonce2, work2): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 212312414);
		let hotkey_account_id2 = 3;
		let coldkey_account_id2 = 669;
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id2), netuid, block_number, nonce2, work2, hotkey_account_id2, coldkey_account_id2));
	});
}

#[test]
fn test_registration_get_neuron_metadata() {
	new_test_ext().execute_with(|| {
		let netuid: u16 = 1;
		let block_number: u64 = 0;
		let tempo: u16 = 13;
		let (nonce0, work0): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 3942084);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;

		add_network(netuid, tempo, 0);

		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce0, work0, hotkey_account_id, coldkey_account_id));
		//
		//let neuron_id = ParatensorModule::get_uid_for_net_and_hotkey(netuid, &hotkey_account_id);
		let neuron_uid = ParatensorModule::get_uid_for_net_and_hotkey( netuid, &hotkey_account_id ).unwrap();
		let neuron: AxonMetadataOf = ParatensorModule::get_neuron_metadata(netuid, neuron_uid);
		assert_eq!(neuron.ip, 0);
		assert_eq!(neuron.version, 0);
		assert_eq!(neuron.port, 0);
	});
}

#[test]
fn test_registration_add_network_size() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
		let netuid2: u16 = 2;
		let block_number: u64 = 0;
		let (nonce0, work0): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid, block_number, 3942084);
		let (nonce1, work1): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid2, block_number, 11231312312);
		let (nonce2, work2): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid2, block_number, 21813123);
		let hotkey_account_id = 1;
		let coldkey_account_id = 667;

		add_network(netuid, 13, 0);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 0);

		add_network(netuid2, 13, 0);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid2), 0);

		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid, block_number, nonce0, work0, hotkey_account_id, coldkey_account_id));
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 1);
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid), 1);


		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(hotkey_account_id), netuid2, block_number, nonce1, work1, hotkey_account_id, coldkey_account_id));
		assert_ok!(ParatensorModule::register(<<Test as Config>::Origin>::signed(2), netuid2, block_number, nonce2, work2, 2, coldkey_account_id));
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid2), 2);
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid2), 2);
	});
}


#[test]
fn test_full_pass_through() {
	new_test_ext().execute_with(|| {

		// Create 3 networks.
        let netuid0: u16 = 0;
		let netuid1: u16 = 1;
		let netuid2: u16 = 2;
		
		// With 3 tempos
		let tempo0: u16 = 2;
		let tempo1: u16 = 2;
		let tempo2: u16 = 2;
		
		// Create 3 keys.
		let hotkey0: u64 = 0;
		let hotkey1: u64 = 1;
		let hotkey2: u64 = 2;

		// With 3 different coldkeys.
		let coldkey0: u64 = 0;
		let coldkey1: u64 = 1;
		let coldkey2: u64 = 2;

		// Add the 3 networks.
		add_network( netuid0, tempo0, 0 );
		add_network( netuid1, tempo1, 0 );
		add_network( netuid2, tempo2, 0 );

		// Check their tempo.
		assert_eq!(ParatensorModule::get_tempo(netuid0), tempo0);
        assert_eq!(ParatensorModule::get_tempo(netuid1), tempo1);
        assert_eq!(ParatensorModule::get_tempo(netuid2), tempo2);

		// Check their emission value.
        assert_eq!(ParatensorModule::get_emission_value(netuid0), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid2), 0);

		// Set their max allowed uids.
		ParatensorModule::set_max_allowed_uids( netuid0, 2 );
		ParatensorModule::set_max_allowed_uids( netuid1, 2 );
		ParatensorModule::set_max_allowed_uids( netuid2, 2 );

		// Check their max allowed.
		assert_eq!( ParatensorModule::get_max_allowed_uids( netuid0 ), 2 );
		assert_eq!( ParatensorModule::get_max_allowed_uids( netuid0 ), 2 );
		assert_eq!( ParatensorModule::get_max_allowed_uids( netuid0 ), 2 );
		
		// Set the max registration per block.
		ParatensorModule::set_max_registrations_per_block( netuid0, 3 );
		ParatensorModule::set_max_registrations_per_block( netuid1, 3 );
		ParatensorModule::set_max_registrations_per_block( netuid2, 3 );
		assert_eq!( ParatensorModule::get_max_registratations_per_block(netuid0), 3 );
		assert_eq!( ParatensorModule::get_max_registratations_per_block(netuid1), 3 );
		assert_eq!( ParatensorModule::get_max_registratations_per_block(netuid2), 3 );

		// Check that no one has registered yet.
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid0), 0);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid1), 0);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid2), 0);

		// Registered the keys to all networks.
    	register_ok_neuron( netuid0, hotkey0, coldkey0, 39420842 );
    	register_ok_neuron( netuid0, hotkey1, coldkey1, 12412392 );
		register_ok_neuron( netuid1, hotkey0, coldkey0, 21813123 );
    	register_ok_neuron( netuid1, hotkey1, coldkey1, 25755207 );
		register_ok_neuron( netuid2, hotkey0, coldkey0, 251232207 );
    	register_ok_neuron( netuid2, hotkey1, coldkey1, 159184122 );

		// Check the number of registrations.
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid0), 2);
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid1), 2);
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid2), 2);

		// Get the number of uids in each network.
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid0), 2);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid1), 2);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid2), 2);

		// Check the uids exist.
		assert!( ParatensorModule::is_uid_exist_on_network( netuid0, 0) );
		assert!( ParatensorModule::is_uid_exist_on_network( netuid1, 0) );
		assert!( ParatensorModule::is_uid_exist_on_network( netuid2, 0) );

		// Check the other exists.
		assert!( ParatensorModule::is_uid_exist_on_network( netuid0, 1) );
		assert!( ParatensorModule::is_uid_exist_on_network( netuid1, 1) );
		assert!( ParatensorModule::is_uid_exist_on_network( netuid2, 1) );

		// Get the hotkey under each uid.
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid0, 0).unwrap(), hotkey0 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid1, 0).unwrap(), hotkey0 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid2, 0).unwrap(), hotkey0 );

		// Get the hotkey under the other uid.
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid0, 1).unwrap(), hotkey1 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid1, 1).unwrap(), hotkey1 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid2, 1).unwrap(), hotkey1 );

		// Check for replacement.
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid0), 2);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid1), 2);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid2), 2);

		// Register the 3rd hotkey.
		register_ok_neuron( netuid0, hotkey2, coldkey2, 59420842 );
		register_ok_neuron( netuid1, hotkey2, coldkey2, 31813123 );
		register_ok_neuron( netuid2, hotkey2, coldkey2, 451232207 );

		// Check for replacement.
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid0), 2);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid1), 2);
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid2), 2);

		// Check the registration counters.
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid0), 3);
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid1), 3);
		assert_eq!(ParatensorModule::get_registrations_this_interval(netuid2), 3);

		// Check the hotkeys are expected.
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid0, 0 ).unwrap(), hotkey2 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid1, 0 ).unwrap(), hotkey2 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid2, 0 ).unwrap(), hotkey2 );
	});
}

#[test]
fn test_bulk_register() {
	new_test_ext().execute_with(|| {
		// Add a network.
		let netuid: u16 = 0;
        add_network(netuid, 0, 0);

		// Bulk values.
		let hotkeys: Vec<u64> = vec![ 0,1,2,3,4,5,6,7,8,9,10 ];
		let coldkeys: Vec<u64> = vec![ 0,1,2,3,4,5,6,7,8,9,10 ];
	
		// Call bulk register with false sudo.
		assert_eq!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::signed(0), netuid, hotkeys.clone(), coldkeys.clone()), Err(DispatchError::BadOrigin.into()) );

		// Lets register some other nodes normally.
		ParatensorModule::set_max_allowed_uids( netuid, 2 );
		register_ok_neuron( netuid, 11, 11, 39420842 );
    	register_ok_neuron( netuid, 12, 12, 12412392 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, 0).unwrap(), 11 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, 1).unwrap(), 12 );
		assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 2);

		// Lets try to bulk register on a non existent network.
		assert_eq!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 1, hotkeys.clone(), coldkeys.clone()), Err(Error::<Test>::NetworkDoesNotExist.into()) );
		
		// Lets try to bulk register with hot.len() != cold.len()
		assert_eq!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 0, vec![ 0,1,2,3 ], vec![ 0,1,2,3,4 ]), Err(Error::<Test>::WeightVecNotEqualSize.into()) );

		// Lets try to bulk register too many uids. (there are only 2 slots.)
		assert_eq!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 0, vec![ 0,1,2,3,4 ], vec![ 0,1,2,3,4 ]), Err(Error::<Test>::NotSettingEnoughWeights.into()) );

		// Lets try to bulk register duplicate hotkeys.
		assert_eq!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 0, vec![ 0, 0 ], vec![ 0,1 ]), Err(Error::<Test>::DuplicateUids.into()) );

		// Lets try to bulk register on a coldkey pair which is already taken.
		assert_eq!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 0, vec![ 11,12 ], vec![ 0,1 ]), Err(Error::<Test>::NonAssociatedColdKey.into()) );

		// Lets register bulk successfully (0,1) --> (11, 12)
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, 0).unwrap(), 11 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, 1).unwrap(), 12 );
		assert_ok!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 0, vec![ 0,1 ], vec![ 0,1 ]) );

		// Check uids are correct.
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, 0).unwrap(), 0 );
		assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, 1).unwrap(), 1 );
		
		// Check coldkey -- hotkey pairing.
		assert_eq!( ParatensorModule::get_owning_coldkey_for_hotkey( &0 ), 0 );
		assert_eq!( ParatensorModule::get_owning_coldkey_for_hotkey( &1 ), 1 );

		// Lets register bulk a huge set.
		let hotkeys: Vec<u64> = (0..4096).collect();
		let coldkeys: Vec<u64> = (0..4096).collect();
		ParatensorModule::set_max_allowed_uids( netuid, 4096 );
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 2 );
		assert_ok!( ParatensorModule::sudo_bulk_register(<<Test as Config>::Origin>::root(), 0, hotkeys.clone(), coldkeys.clone() ) );
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 4096 );
		for i in 0..4096 {
			assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, i as u16 ).unwrap(), i as u64 );
			assert_eq!( ParatensorModule::get_owning_coldkey_for_hotkey( &(i as u64) ), i as u64  );
		}

	});
}

#[test]
fn test_network_connection_requirement() {
	new_test_ext().execute_with(|| {
		// Add a networks and connection requirements.
		let netuid_a: u16 = 0;
		let netuid_b: u16 = 1;
		add_network(netuid_a, 10, 0);
		add_network(netuid_b, 10, 0);

		// Bulk values.
		let hotkeys: Vec<u64> = vec![ 0,1,2,3,4,5,6,7,8,9,10 ];
		let coldkeys: Vec<u64> = vec![ 0,1,2,3,4,5,6,7,8,9,10 ];

		// Add a connection requirement between the A and B. A requires B.
		ParatensorModule::add_connection_requirement( netuid_a, netuid_b, u16::MAX );
		ParatensorModule::set_max_registrations_per_block( netuid_a, 10 ); // Enough for the below tests.
		ParatensorModule::set_max_registrations_per_block( netuid_b, 10 ); // Enough for the below tests.
		ParatensorModule::set_max_allowed_uids( netuid_a, 10 ); // Enough for the below tests.
		ParatensorModule::set_max_allowed_uids( netuid_b, 10 ); // Enough for the below tests.

		// Attempt registration on A fails because the hotkey is not registered on network B.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_a, 0, 3942084);
		assert_eq!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[0] ), netuid_a, 0, nonce, work, hotkeys[0], coldkeys[0]), Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into()) );
		
		// Attempt registration on B passes because there is no exterior requirement.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_b, 0, 5942084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[0] ), netuid_b, 0, nonce, work, hotkeys[0], coldkeys[0]) );

		// Attempt registration on A passes because this key is in the top 100 of keys on network B.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_a, 0, 6942084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[0] ), netuid_a, 0, nonce, work, hotkeys[0], coldkeys[0]) );

		// Lets attempt the key registration on A. Fails because we are not in B.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_a, 0, 6942084);
		assert_eq!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[1] ), netuid_a, 0, nonce, work, hotkeys[1], coldkeys[1]), Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into()) );

		// Lets register the next key on B. Passes, np.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_b, 0, 7942084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[1] ), netuid_b, 0, nonce, work, hotkeys[1], coldkeys[1]) );

		// Lets make the connection requirement harder. Top 0th percentile.
		ParatensorModule::add_connection_requirement( netuid_a, netuid_b, 0 );

		// Attempted registration passes because the prunning score for hotkey_1 is the top keys on network B.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_a, 0, 8942084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[1] ), netuid_a, 0, nonce, work, hotkeys[1], coldkeys[1]) );

		// Lets register key 3 with lower prunning score.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_b, 0, 9942084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[2] ), netuid_b, 0, nonce, work, hotkeys[2], coldkeys[2]) );
		ParatensorModule::set_pruning( netuid_b, ParatensorModule::get_uid_for_net_and_hotkey( netuid_b, &hotkeys[2] ).unwrap(), 0); // Set prunning score to 0.
		ParatensorModule::set_pruning( netuid_b, ParatensorModule::get_uid_for_net_and_hotkey( netuid_b, &hotkeys[1] ).unwrap(), 0); // Set prunning score to 0.
		ParatensorModule::set_pruning( netuid_b, ParatensorModule::get_uid_for_net_and_hotkey( netuid_b, &hotkeys[0] ).unwrap(), 0); // Set prunning score to 0.

		// Lets register key 4 with higher prunining score.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_b, 0, 10142084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[3] ), netuid_b, 0, nonce, work, hotkeys[3], coldkeys[3]) );
		ParatensorModule::set_pruning( netuid_b, ParatensorModule::get_uid_for_net_and_hotkey( netuid_b, &hotkeys[2] ).unwrap(), 1); // Set prunning score to 1.

		// Attempted register of key 3 fails because of bad prunning score on B.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_a, 0, 11142084);
		assert_eq!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[2] ), netuid_a, 0, nonce, work, hotkeys[2], coldkeys[2]), Err(Error::<Test>::DidNotPassConnectedNetworkRequirement.into()) );	

		// Attempt to register key 4 passes because of best prunning score on B.
		let (nonce, work): (u64, Vec<u8>) = ParatensorModule::create_work_for_block_number( netuid_b, 0, 12142084);
		assert_ok!( ParatensorModule::register(<<Test as Config>::Origin>::signed( hotkeys[3] ), netuid_a, 0, nonce, work, hotkeys[3], coldkeys[3]) );
	});
}