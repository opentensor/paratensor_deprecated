use crate::{mock::*};
use pallet_nakamoto::{Error};
use frame_system::{Config};
use frame_support::assert_ok;
use frame_support::sp_runtime::DispatchError;

mod mock;
mod helpers;

#[test]
fn test_empty_network() {
	new_test_ext().execute_with(|| {
        helpers::assert_u16_vec_eq( &NakamotoModule::get_rank( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_trust( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_incentive( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_consensus( 0 ), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_dividends( 0 ), &vec![] );
        helpers::assert_u64_vec_eq( &NakamotoModule::get_emission( 0 ), &vec![] );
    });
}

#[test]
fn test_set_network_conensus() {
	new_test_ext().execute_with(|| {
        NakamotoModule::set_rank( 0, vec![0,1,2,3] );
        NakamotoModule::set_trust( 0, vec![0,1,2,3] );
        NakamotoModule::set_incentive( 0, vec![0,1,2,3] );
        NakamotoModule::set_consensus( 0, vec![0,1,2,3] );
        NakamotoModule::set_dividends( 0, vec![0,1,2,3] );
        NakamotoModule::set_emission( 0, vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_rank( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_trust( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_incentive( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_consensus( 0 ), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_dividends( 0 ), &vec![0,1,2,3] );
        helpers::assert_u64_vec_eq( &NakamotoModule::get_emission( 0 ), &vec![0,1,2,3] );
        NakamotoModule::set_rank_for_uid( 0, 0, 4 );
        NakamotoModule::set_trust_for_uid( 0, 0, 4 );
        NakamotoModule::set_incentive_for_uid( 0, 0, 4 );
        NakamotoModule::set_consensus_for_uid( 0, 0, 4 );
        NakamotoModule::set_dividends_for_uid( 0, 0, 4 );
        NakamotoModule::set_emission_for_uid( 0, 0, 4 );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_rank( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_trust( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_incentive( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_consensus( 0 ), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_dividends( 0 ), &vec![4,1,2,3] );
        helpers::assert_u64_vec_eq( &NakamotoModule::get_emission( 0 ), &vec![4,1,2,3] );
    });
}