use pallet_paratensor::{Error};
use frame_support::{assert_ok};
use frame_system::Config;
//use mock::*;
use crate::{mock::*};
//use frame_support::sp_runtime::DispatchError;
use frame_support::dispatch::{GetDispatchInfo, DispatchInfo};
use frame_support::weights::{DispatchClass, Pays};

mod mock;

/********************************************
	subscribing::subscribe() tests
*********************************************/
#[test]
fn test_subscribe_ok_dispatch_info_ok() {
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
fn test_difficulty() {
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
		assert_eq!(ParatensorModule::get_neuron_count(netuid), 1);

		//check if hotkey is added to the Hotkeys
		assert_eq!(ParatensorModule::get_coldkey_for_hotkey(hotkey_account_id), coldkey_account_id);

		//check if coldkey is added to coldkeys
		assert_eq!(ParatensorModule::get_hotkey_for_coldkey(coldkey_account_id), hotkey_account_id);

		// Check the list of neworks that uid has registered 
		let subnets = ParatensorModule::get_subnets_for_hotkey(hotkey_account_id);
		assert_eq!(subnets.contains(&netuid), true);

		// Check if the neuron has added to the Keys
		let neuron_uid: u16 = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, hotkey_account_id);
		assert_eq!(ParatensorModule::get_hotkey_for_net_and_neuron(netuid, neuron_uid), hotkey_account_id);

		// Check if neuron has added to Uids
        assert_eq!(ParatensorModule::get_neuron_for_net_and_hotkey(netuid, hotkey_account_id), neuron_uid);

		// Check if the balance of this hotkey account for this subnetwork == 0
		assert_eq!(ParatensorModule::get_neuron_stake_for_subnetwork(netuid, neuron_uid), 0);
	});
}