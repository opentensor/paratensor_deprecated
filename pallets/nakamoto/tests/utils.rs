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
        helpers::assert_u16_vec_eq( &NakamotoModule::get_rank(), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_trust(), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_incentive(), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_consensus(), &vec![] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_dividends(), &vec![] );
        helpers::assert_u64_vec_eq( &NakamotoModule::get_emission(), &vec![] );
    });
}

#[test]
fn test_set_network_conensus() {
	new_test_ext().execute_with(|| {
        NakamotoModule::set_rank( vec![0,1,2,3] );
        NakamotoModule::set_trust( vec![0,1,2,3] );
        NakamotoModule::set_incentive( vec![0,1,2,3] );
        NakamotoModule::set_consensus( vec![0,1,2,3] );
        NakamotoModule::set_dividends( vec![0,1,2,3] );
        NakamotoModule::set_emission( vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_rank(), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_trust(), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_incentive(), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_consensus(), &vec![0,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_dividends(), &vec![0,1,2,3] );
        helpers::assert_u64_vec_eq( &NakamotoModule::get_emission(), &vec![0,1,2,3] );
        NakamotoModule::set_rank_for_uid( 0, 4 );
        NakamotoModule::set_trust_for_uid( 0, 4 );
        NakamotoModule::set_incentive_for_uid( 0, 4 );
        NakamotoModule::set_consensus_for_uid( 0, 4 );
        NakamotoModule::set_dividends_for_uid( 0, 4 );
        NakamotoModule::set_emission_for_uid( 0, 4 );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_rank(), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_trust(), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_incentive(), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_consensus(), &vec![4,1,2,3] );
        helpers::assert_u16_vec_eq( &NakamotoModule::get_dividends(), &vec![4,1,2,3] );
        helpers::assert_u64_vec_eq( &NakamotoModule::get_emission(), &vec![4,1,2,3] );
    });
}