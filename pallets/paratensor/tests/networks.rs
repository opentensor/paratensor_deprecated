mod mock;
use mock::*;
use pallet_paratensor::{Error};
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};
use frame_support::{assert_ok};
use sp_runtime::DispatchError;
use frame_system::Config;
use sp_core::U256;
use frame_support::{sp_std::vec};

/*TO DO SAM: write test for LatuUpdate after it is set */

// --- add network tests ----
#[test]
fn test_add_network_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let modality = 0;

		let call = Call::ParatensorModule(ParatensorCall::sudo_add_network{netuid, modality});

		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Operational,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_add_network() {
	new_test_ext().execute_with(|| {

        let modality = 0;
        //
		add_network(10, modality);
        assert_eq!(ParatensorModule::get_number_of_subnets(), 1);
        //
        add_network( 20, modality);
        assert_eq!(ParatensorModule::get_number_of_subnets(), 2);

	});
}

// --- remove network tests ---
#[test]
fn test_remove_priority_for_network() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
        let weights_keys: Vec<u16> = vec![];
		let weight_values: Vec<u16> = vec![];

		add_network(netuid, 0);
        //
        register_ok_neuron( 1, 55, 66, 0);
        let neuron_id = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &55);
        //
        ParatensorModule::set_priority_for_neuron(netuid, neuron_id, 1);
        assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_id), 1);
        //
        assert_ok!(ParatensorModule::set_weights(Origin::signed(55), netuid, weights_keys, weight_values));
        //
        assert_eq!(ParatensorModule::get_priority_for_neuron(netuid, neuron_id), 0);

	});
}

#[test]
fn test_clear_min_allowed_weight_for_network() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
        let min_allowed_weight = 2;

        add_network(netuid, 0);
        //
		register_ok_neuron( 1, 55, 66, 0);
        //
        ParatensorModule::set_min_allowed_weights(netuid, min_allowed_weight);
        assert_eq!(ParatensorModule::get_min_allowed_weights(netuid), 2);
        //
        assert_ok!(ParatensorModule::do_remove_network(<<Test as Config>::Origin>::root(), netuid));
        //
        assert_eq!(ParatensorModule::get_min_allowed_weights(netuid), 0);

	});
}

#[test]
fn test_remove_uid_for_network() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
        let mut result = 00;

        add_network(netuid, 0);
        //
		register_ok_neuron( 1, 55, 66, 0);
        let neuron_id = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &55);
        assert_eq!(neuron_id, 0);
        //
        register_ok_neuron( 1, 56, 67, 300000);
        let neuron_id = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &56);
        assert_eq!(neuron_id, 1);
        //
        assert_ok!(ParatensorModule::do_remove_network(<<Test as Config>::Origin>::root(), netuid));
        //
        result = ParatensorModule::get_neuron_for_net_and_hotkey(netuid, &55);
        assert_eq!(result, 00);

	});
}

#[test]
fn test_remove_difficulty_for_network() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
        let difficulty: u64 = 10;

        add_network(netuid, 0);
        //
		register_ok_neuron( 1, 55, 66, 0);
        //
        assert_ok!(ParatensorModule::sudo_set_difficulty(<<Test as Config>::Origin>::root(), netuid, difficulty));
        assert_eq!(ParatensorModule::get_difficulty_as_u64(netuid), difficulty);
        //
        assert_ok!(ParatensorModule::do_remove_network(<<Test as Config>::Origin>::root(), netuid));
        //
        assert_eq!(ParatensorModule::get_difficulty_as_u64(netuid), 10000);

	});
}

#[test]
fn test_remove_network_for_all_hotkeys() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;
        let test: Vec<u16>= vec![];

        add_network(netuid, 0);
        //
		register_ok_neuron( 1, 55, 66, 0);
        register_ok_neuron( 1, 77, 88, 65536);
        //
        assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 2);
        assert_ne!(ParatensorModule::get_subnets_for_hotkey(55), test); 
        assert_ne!(ParatensorModule::get_subnets_for_hotkey(77), test); 
        //
        assert_ok!(ParatensorModule::do_remove_network(<<Test as Config>::Origin>::root(), netuid));
        //
        assert_eq!(ParatensorModule::get_subnetwork_n(netuid), 0);
        assert_eq!(ParatensorModule::get_subnets_for_hotkey(55), test); 
        assert_eq!(ParatensorModule::get_subnets_for_hotkey(77), test); 
	});
}

#[test]
fn test_network_set_default_value_for_other_parameters() {
	new_test_ext().execute_with(|| {

        let netuid: u16 = 1;

        add_network(netuid, 0);
        //
        assert_eq!(ParatensorModule::get_min_allowed_weights(netuid), 0);
        assert_eq!(ParatensorModule::get_emission_ratio(netuid), 0);
        assert_eq!(ParatensorModule::get_max_weight_limit(netuid), u16::MAX);
        //maxAllowedMaxMinRatio
        assert_eq!(ParatensorModule::get_difficulty_as_u64(netuid), 10000);
		assert_eq!(ParatensorModule::get_immunity_period(netuid), 2);
        
	});
}
