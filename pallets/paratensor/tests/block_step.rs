mod mock;
use mock::*;

#[test]
fn test_block_step_multi(){
    new_test_ext().execute_with(|| { 
    // This test show cases the entire process of network expansion, epochs, tempo and difficulty adjustment.

    // First assert all defaults.
    let netuid: u16 = 0;
    let tempo: u16 = 1;
    let modality: u16 = 1;
    add_network( netuid, tempo, modality );
    assert_eq!( ParatensorModule::get_number_of_subnets(), 1 ); // There is a single network.
    assert_eq!( ParatensorModule::get_subnetwork_n( netuid ), 0 ); // Network size is zero.
    assert_eq!( ParatensorModule::get_rho( netuid ), 10 );
    assert_eq!( ParatensorModule::get_tempo( netuid ), 1 );
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

    // Lets step a block.
    // Here there is no emission on any network so pending emission is not incremented.
    assert_eq!( ParatensorModule::get_emission_value( netuid ), 0 );
    step_block(1);
    assert_eq!( ParatensorModule::get_pending_emission( netuid ), 0 );

    // Lets set the block emission for this network. It will get all the emission.
    ParatensorModule::set_emission_for_network( netuid, ParatensorModule::get_block_emission() );
    assert_eq!( ParatensorModule::get_emission_value( netuid ), ParatensorModule::get_block_emission()  );
    step_block( 1 );
    assert_eq!( ParatensorModule::get_pending_emission( netuid ), ParatensorModule::get_block_emission()  );
    });
}