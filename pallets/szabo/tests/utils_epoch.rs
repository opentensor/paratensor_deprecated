use crate::{mock::*};
use frame_support::assert_ok;

mod mock;
mod helpers;


#[test]
fn test_empty_network() {
	new_test_ext().execute_with(|| {
        // Test zero network.
        helpers::assert_u16_vec_eq( &SzaboModule::get_rank( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_trust( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_incentive( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_consensus( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_dividends( 0 ), &vec![] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_emission( 0 ), &vec![] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 0 ), &vec![] );

        // Test network one.
        helpers::assert_u16_vec_eq( &SzaboModule::get_rank( 1 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_trust( 1 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_incentive( 1 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_consensus( 1 ), &vec![] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_dividends( 1 ), &vec![] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_emission( 1 ), &vec![] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 1 ), &vec![] );
    });
}

#[test]
fn test_set_network_conensus() {
	new_test_ext().execute_with(|| {

        // Test zero network.
        SzaboModule::set_rank_from_vector( 0, vec![0,1,2,3] );
        SzaboModule::set_trust_from_vector( 0, vec![0,1,2,3] );
        SzaboModule::set_incentive_from_vector( 0, vec![0,1,2,3] );
        SzaboModule::set_consensus_from_vector( 0, vec![0,1,2,3] );
        SzaboModule::set_dividends_from_vector( 0, vec![0,1,2,3] );
        SzaboModule::set_emission_from_vector( 0, vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_rank( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_trust( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_incentive( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_consensus( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_dividends( 0 ), &vec![0,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_emission( 0 ), &vec![0,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 0 ), &vec![0,0,0,0] );
        SzaboModule::set_rank_for_uid( 0, 0, 4 );
        SzaboModule::set_trust_for_uid( 0, 0, 4 );
        SzaboModule::set_incentive_for_uid( 0, 0, 4 );
        SzaboModule::set_consensus_for_uid( 0, 0, 4 );
        SzaboModule::set_dividends_for_uid( 0, 0, 4 );
        SzaboModule::set_emission_for_uid( 0, 0, 4 );
        helpers::assert_u16_vec_eq( &SzaboModule::get_rank( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_trust( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_incentive( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_consensus( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_dividends( 0 ), &vec![4,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_emission( 0 ), &vec![4,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 0 ), &vec![0,0,0,0] );

        // Test network one
        SzaboModule::set_rank_from_vector( 1, vec![5,1,2,3] );
        SzaboModule::set_trust_from_vector( 1, vec![5,1,2,3] );
        SzaboModule::set_incentive_from_vector( 1, vec![5,1,2,3] );
        SzaboModule::set_consensus_from_vector( 1, vec![5,1,2,3] );
        SzaboModule::set_dividends_from_vector( 1, vec![5,1,2,3] );
        SzaboModule::set_emission_from_vector( 1, vec![5,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_rank( 1 ), &vec![5,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_trust( 1 ), &vec![5,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_incentive( 1 ), &vec![5,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_consensus( 1 ), &vec![5,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_dividends( 1 ), &vec![5,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_emission( 1 ), &vec![5,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 1 ), &vec![0,0,0,0] );

        SzaboModule::set_rank_for_uid( 1, 0, 4 );
        SzaboModule::set_trust_for_uid( 1, 0, 4 );
        SzaboModule::set_incentive_for_uid( 1, 0, 4 );
        SzaboModule::set_consensus_for_uid( 1, 0, 4 );
        SzaboModule::set_dividends_for_uid( 1, 0, 4 );
        SzaboModule::set_emission_for_uid( 1, 0, 4 );
        helpers::assert_u16_vec_eq( &SzaboModule::get_rank( 1 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_trust( 1 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_incentive( 1 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_consensus( 1 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &SzaboModule::get_dividends( 1 ), &vec![4,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_emission( 1 ), &vec![4,1,2,3] );
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 1 ), &vec![0,0,0,0] );
    });
}


#[test]
fn test_stake_vector() {
	new_test_ext().execute_with(|| {
        // Create global account
        SzaboModule::add_global_account( &0, &0 );
		assert_eq!( SzaboModule::get_global_n(), 1 );

        // Add balance to staking account.
		SzaboModule::add_balance_to_coldkey_account( &0, 100000 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(0), 0, 100000));
		assert_eq!(SzaboModule::has_enough_stake(&0, 100000), true);
		assert_eq!(SzaboModule::get_total_stake(), 100000);

        // Add account to subnetwork.
        SzaboModule::add_subnetwork_account( 0, 0, &0 );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 0, &0 ) );

        // Check staking account is pulled over.
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 0 ), &vec![ 100000 ] );

        // Create another global account
        SzaboModule::add_global_account( &1, &1 );
        assert_eq!( SzaboModule::get_global_n(), 2 );

        // Add balance to staking account.
		SzaboModule::add_balance_to_coldkey_account( &1, 200000 );
		assert_ok!(SzaboModule::add_stake(Origin::signed(1), 1, 200000));
		assert_eq!(SzaboModule::has_enough_stake(&1, 200000), true);
		assert_eq!(SzaboModule::get_total_stake(), 300000);

        // Add account to subnetwork.
        SzaboModule::add_subnetwork_account( 0, 1, &1 );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 0, &1 ) );

        // Check staking account is pulled over.
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 0 ), &vec![ 100000, 200000 ] );

        // Check other network
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 1 ), &vec![] );

        // Add the accounts to other subnetwork.
        SzaboModule::add_subnetwork_account( 1, 0, &0 );
        SzaboModule::add_subnetwork_account( 1, 1, &1 );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 1, &0 ) );
        assert!( SzaboModule::is_hotkey_subnetwork_active( 1, &1 ) );

        // Check staking account is pulled over to the other network.
        helpers::assert_u64_vec_eq( &SzaboModule::get_stake( 1 ), &vec![ 100000, 200000 ] );
    });
}