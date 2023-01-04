use frame_support::{assert_ok};
use frame_system::Config;
mod mock;
use mock::*;
use frame_support::sp_runtime::DispatchError;
use pallet_paratensor::{Error};


#[test]
fn test_defaults() {
    new_test_ext().execute_with(|| {
        let netuid = 0;
        add_network(netuid, 10, 0);
        assert_eq!( ParatensorModule::get_number_of_subnets(), 1 ); // There is a single network.
        assert_eq!( ParatensorModule::get_subnetwork_n( netuid ), 0 ); // Network size is zero.
        assert_eq!( ParatensorModule::get_rho( netuid ), 10 );
        assert_eq!( ParatensorModule::get_tempo( netuid ), 10 );
        assert_eq!( ParatensorModule::get_kappa( netuid ), 32_767 );
        assert_eq!( ParatensorModule::get_min_difficulty( netuid ), 1 );
        assert_eq!( ParatensorModule::get_max_difficulty( netuid ), u64::MAX );
        assert_eq!( ParatensorModule::get_difficulty_as_u64( netuid ), 10000 );
        assert_eq!( ParatensorModule::get_immunity_period( netuid ), 2 );
        assert_eq!( ParatensorModule::get_emission_value( netuid ), 0 );
        assert_eq!( ParatensorModule::get_activity_cutoff( netuid ), 5000 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid ), 0 );
        assert_eq!( ParatensorModule::get_max_weight_limit( netuid ), u16::MAX );
        assert_eq!( ParatensorModule::get_max_allowed_uids( netuid ), 2 );
        assert_eq!( ParatensorModule::get_min_allowed_weights( netuid ), 0 );
        assert_eq!( ParatensorModule::get_adjustment_interval( netuid ), 100 );
        assert_eq!( ParatensorModule::get_bonds_moving_average( netuid ), 500_000 );
        assert_eq!( ParatensorModule::get_validator_batch_size( netuid ), 10 );
        assert_eq!( ParatensorModule::get_last_adjustment_block( netuid ), 0 );
        assert_eq!( ParatensorModule::get_last_mechanism_step_block( netuid ), 0 );
        assert_eq!( ParatensorModule::get_blocks_since_last_step( netuid ), 0 );
        assert_eq!( ParatensorModule::get_registrations_this_block( netuid ), 0 );
        assert_eq!( ParatensorModule::get_validator_epochs_per_reset( netuid ), 10 );
        assert_eq!( ParatensorModule::get_validator_sequence_length( netuid ), 10 );
        assert_eq!( ParatensorModule::get_validator_exclude_quantile( netuid ), 10 );
        assert_eq!( ParatensorModule::get_registrations_this_interval( netuid ), 0 );
        assert_eq!( ParatensorModule::get_max_registratations_per_block( netuid ), 3 );
        assert_eq!( ParatensorModule::get_target_registrations_per_interval( netuid ), 2 );
    });
}

#[test]
fn test_sudo_set_default_take() {
	new_test_ext().execute_with(|| {
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_default_take();
		assert_eq!( ParatensorModule::sudo_set_default_take(<<Test as Config>::Origin>::signed(0), to_be_set), Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::get_default_take(), init_value);
        assert_ok!( ParatensorModule::sudo_set_default_take(<<Test as Config>::Origin>::root(), to_be_set) );
        assert_eq!( ParatensorModule::get_default_take(), to_be_set);
    });
}


#[test]
fn test_sudo_set_weights_version_key() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        let init_value: u64 = ParatensorModule::get_weights_version_key( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_weights_version_key(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_weights_version_key(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_weights_version_key(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_weights_version_key(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_weights_version_key(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_weights_set_rate_limit() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        let init_value: u64 = ParatensorModule::get_weights_set_rate_limit( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_weights_set_rate_limit(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_weights_set_rate_limit(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_weights_set_rate_limit(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_weights_set_rate_limit(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_weights_set_rate_limit(netuid), to_be_set);
    });
}


#[test]
fn test_sudo_set_adjustment_interval() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_adjustment_interval( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_adjustment_interval(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_adjustment_interval(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_adjustment_interval(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_adjustment_interval(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_adjustment_interval(netuid), to_be_set);
    });
}


#[test]
fn test_sudo_set_validator_exclude_quantile() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_validator_exclude_quantile( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_validator_exclude_quantile(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_validator_exclude_quantile(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_validator_exclude_quantile(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_validator_exclude_quantile(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_validator_exclude_quantile(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_weight_limit() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_max_weight_limit( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_max_weight_limit(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_max_weight_limit(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_max_weight_limit(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_max_weight_limit(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_max_weight_limit(netuid), to_be_set);
    });
}


#[test]
fn test_sudo_set_immunity_period() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_immunity_period( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_immunity_period(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_immunity_period(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_immunity_period(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_immunity_period(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_immunity_period(netuid), to_be_set);
    });
}


#[test]
fn test_sudo_set_validator_epochs_per_reset() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_validator_epochs_per_reset( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_validator_epochs_per_reset(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_validator_epochs_per_reset(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_validator_epochs_per_reset(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_validator_epochs_per_reset(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_validator_epochs_per_reset(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_validator_sequence_length() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_validator_sequence_length( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_validator_sequence_length(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_validator_sequence_length(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_validator_sequence_length(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_validator_sequence_length(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_validator_sequence_length(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_validator_batch_size() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_validator_batch_size( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_validator_batch_size(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_validator_batch_size(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_validator_batch_size(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_validator_batch_size(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_validator_batch_size(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_min_allowed_weights() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_min_allowed_weights( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_min_allowed_weights(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_min_allowed_weights(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_min_allowed_weights(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_min_allowed_weights(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_min_allowed_weights(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_max_allowed_uids() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_max_allowed_uids( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_max_allowed_uids(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_max_allowed_uids(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_max_allowed_uids(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_max_allowed_uids(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_max_allowed_uids(netuid), to_be_set);
    });
}


#[test]
fn test_sudo_set_kappa() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_kappa( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_kappa(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_kappa(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_kappa(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_kappa(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_kappa(netuid), to_be_set);
    });
}
        

#[test]
fn test_sudo_set_rho() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_rho( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_rho(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_rho(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_rho(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_rho(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_rho(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_activity_cutoff() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_activity_cutoff( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_activity_cutoff(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_activity_cutoff(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_activity_cutoff(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_activity_cutoff(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_activity_cutoff(netuid), to_be_set);
    });
}
        
        
#[test]
fn test_sudo_set_target_registrations_per_interval() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_target_registrations_per_interval( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_target_registrations_per_interval(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_target_registrations_per_interval(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_target_registrations_per_interval(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_target_registrations_per_interval(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_target_registrations_per_interval(netuid), to_be_set);
    });
}
        
#[test]
fn test_sudo_set_difficulty() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        let init_value: u64 = ParatensorModule::get_difficulty_as_u64( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_difficulty(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_difficulty(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_difficulty_as_u64(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_difficulty(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_difficulty_as_u64(netuid), to_be_set);
    });
}
        

#[test]
fn test_sudo_set_max_allowed_validators() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u16 = 10;
        let init_value: u16 = ParatensorModule::get_max_allowed_validators( netuid );
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_max_allowed_validators(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_max_allowed_validators(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_max_allowed_validators(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_max_allowed_validators(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_max_allowed_validators(netuid), to_be_set);
    });
}


#[test]
fn test_sudo_set_bonds_moving_average() {
	new_test_ext().execute_with(|| {
        let netuid: u16 = 1;
        let to_be_set: u64 = 10;
        let init_value: u64 = ParatensorModule::get_bonds_moving_average(netuid);
        add_network(netuid, 10, 0);
		assert_eq!( ParatensorModule::sudo_set_bonds_moving_average(<<Test as Config>::Origin>::signed(0), netuid, to_be_set),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_set_bonds_moving_average(<<Test as Config>::Origin>::root(), netuid + 1, to_be_set), Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::get_bonds_moving_average(netuid), init_value);
        assert_ok!( ParatensorModule::sudo_set_bonds_moving_average(<<Test as Config>::Origin>::root(), netuid, to_be_set) );
        assert_eq!( ParatensorModule::get_bonds_moving_average(netuid), to_be_set);
    });
}

#[test]
fn test_sudo_set_network_connection_requirement() {
	new_test_ext().execute_with(|| {
        let netuid_a: u16 = 1;
        let netuid_b: u16 = 2;
        let requirement: u16 = u16::MAX;
        assert_eq!( ParatensorModule::sudo_add_network_connection_requirement(<<Test as Config>::Origin>::signed(0), netuid_a, netuid_b, requirement),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_add_network_connection_requirement(<<Test as Config>::Origin>::root(), netuid_a, netuid_b, requirement),  Err(Error::<Test>::NetworkDoesNotExist.into()) );
        add_network( netuid_a, 10, 0 );
        assert_eq!( ParatensorModule::sudo_add_network_connection_requirement(<<Test as Config>::Origin>::root(), netuid_a, netuid_a, requirement),  Err(Error::<Test>::InvalidConnectionRequirement.into()) );
        assert_eq!( ParatensorModule::sudo_add_network_connection_requirement(<<Test as Config>::Origin>::root(), netuid_a, netuid_b, requirement),  Err(Error::<Test>::NetworkDoesNotExist.into()) );
        add_network( netuid_b, 10, 0 );
        assert_ok!( ParatensorModule::sudo_add_network_connection_requirement(<<Test as Config>::Origin>::root(), netuid_a, netuid_b, requirement));
        assert_eq!( ParatensorModule::get_network_connection_requirement( netuid_a, netuid_b ), requirement);
        assert_eq!( ParatensorModule::sudo_remove_network_connection_requirement(<<Test as Config>::Origin>::signed(0), netuid_a, netuid_b),  Err(DispatchError::BadOrigin.into()) );
        assert_eq!( ParatensorModule::sudo_remove_network_connection_requirement(<<Test as Config>::Origin>::root(), 5 as u16, 5 as u16),  Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_eq!( ParatensorModule::sudo_remove_network_connection_requirement(<<Test as Config>::Origin>::root(), netuid_a, 5 as u16),  Err(Error::<Test>::NetworkDoesNotExist.into()) );
        assert_ok!( ParatensorModule::sudo_remove_network_connection_requirement(<<Test as Config>::Origin>::root(), netuid_a, netuid_b) );
        assert_eq!( ParatensorModule::network_connection_requirement_exists( netuid_a, netuid_b ), false );
    });
}

/// -------- tests for PendingEmissionValues --------
#[test]
fn test_sudo_test_tempo_pending_emissions_ok() {
	new_test_ext().execute_with(|| {
        let netuid0: u16 = 1;
        let netuid1: u16 = 2;
        let netuid2: u16 = 3;
        let netuid3: u16 = 5;
        let tempo0: u16 = 1;
        let tempo1: u16 = 2;
        let tempo2: u16 = 3;
        let tempo3: u16 = 5;
        add_network(netuid0, tempo0, 0);
		add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);
        add_network(netuid3, tempo3, 0);
        assert_eq!(ParatensorModule::get_tempo(netuid0), tempo0);
        assert_eq!(ParatensorModule::get_tempo(netuid1), tempo1);
        assert_eq!(ParatensorModule::get_tempo(netuid2), tempo2);
        assert_eq!(ParatensorModule::get_tempo(netuid3), tempo3);
        assert_eq!(ParatensorModule::get_emission_value(netuid0), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid2), 0);
        assert_eq!(ParatensorModule::get_emission_value(netuid3), 0);
        let netuids: Vec<u16> = vec![ 1, 2, 3, 5 ]; 
        let emission: Vec<u64> = vec![ 100000000, 400000000, 200000000, 300000000];         
        assert_ok!(ParatensorModule::sudo_set_emission_values(<<Test as Config>::Origin>::root(), netuids, emission ));
        assert_eq!(ParatensorModule::get_emission_value(netuid0), 100000000);
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 400000000);
        assert_eq!(ParatensorModule::get_emission_value(netuid2), 200000000);
        assert_eq!(ParatensorModule::get_emission_value(netuid3), 300000000);
        assert_eq!(ParatensorModule::get_pending_emission(netuid0), 0);
        assert_eq!(ParatensorModule::get_pending_emission(netuid1), 0);
        assert_eq!(ParatensorModule::get_pending_emission(netuid2), 0);
        assert_eq!(ParatensorModule::get_pending_emission(netuid3), 0);
        
    });
}

#[test]
pub fn test_sudo_test_pending_emission_ok() {
    new_test_ext().execute_with(|| {
        let netuid1: u16 = 1;
        let tempo1: u16 = 5;

        let netuid2: u16 = 2;
        let tempo2: u16 = 7;

        let netuids: Vec<u16> = vec![ 1, 2 ]; 
        let emission: Vec<u64> = vec![ 250000000, 750000000];         

        add_network(netuid1, tempo1, 0);
        add_network(netuid2, tempo2, 0);

        assert_ok!(ParatensorModule::sudo_set_emission_values(<<Test as Config>::Origin>::root(), netuids, emission ));
        assert_eq!(ParatensorModule::get_emission_value(netuid1), 250000000);

        step_block(3);

        assert_eq!(ParatensorModule::get_pending_emission(netuid1), 750000000); // 250000000 + 250000000 + 250000000
        assert_eq!(ParatensorModule::get_pending_emission(netuid2), 2250000000); // 750000000 + 750000000 + 750000000
    });
}

