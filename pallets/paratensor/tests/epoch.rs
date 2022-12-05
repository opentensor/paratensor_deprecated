use crate::{mock::*};
#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};
use frame_support::{assert_ok};
use std::time::{Instant};
mod mock;

#[test]
// Test an epoch on an empty graph.
fn test_nill_epoch_paratensor() {
	new_test_ext().execute_with(|| {
        println!( "test_nill_epoch:" );
		ParatensorModule::epoch( 0, 0, true );
	});
}

#[test]
// Test an epoch on a graph with a single item.
fn test_1_graph() {
	new_test_ext().execute_with(|| {
        println!( "test_1_graph:" );
		let netuid: u16 = 0;
		let coldkey: u64 = 0;
		let hotkey: u64 = 0;
		let uid: u16 = 0;
		let stake_amount: u64 = 1;
		ParatensorModule::add_balance_to_coldkey_account( &coldkey, stake_amount as u128 );
 		ParatensorModule::set_stake_for_testing( &hotkey, stake_amount );
		ParatensorModule::add_subnetwork_account( netuid, uid, &hotkey );
		ParatensorModule::increment_subnetwork_n( netuid );
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 1 );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		assert_ok!(ParatensorModule::set_weights(Origin::signed(uid as u64), netuid, vec![ uid as u16 ], vec![ u16::MAX ]));
		// ParatensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( 0, u16::MAX )]); // doesn't set update status
		// ParatensorModule::set_bonds_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]); // rather, bonds are calculated in epoch
		ParatensorModule::epoch( 0, 1_000_000_000, true );
		assert_eq!( ParatensorModule::get_stake_for_hotkey( &hotkey ), stake_amount );
		assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
		assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_emission( netuid, uid ), 0 );  // all self-weight masked out (TODO: decide emission for this case)
	});
}

#[test]
// Test an epoch on a graph with two items.
fn test_10_graph() {
	new_test_ext().execute_with(|| {
        println!( "test_10_graph" );
		// Function for adding a nodes to the graph.
		pub fn add_node( 
				netuid: u16,
				coldkey: u64, 
				hotkey:u64, 
				uid: u16, 
				stake_amount: u64
			){
			println!(
				"+Add net:{:?} coldkey:{:?} hotkey:{:?} uid:{:?} stake_amount: {:?} subn: {:?}", 
				netuid,
				coldkey,
				hotkey,
				uid,
				stake_amount,
				ParatensorModule::get_subnetwork_n(netuid),
			);
			ParatensorModule::add_balance_to_coldkey_account( &coldkey, stake_amount as u128 );
			ParatensorModule::set_stake_for_testing( &hotkey, stake_amount );
			ParatensorModule::add_subnetwork_account( netuid, uid, &hotkey );
		   	ParatensorModule::increment_subnetwork_n( netuid );
			assert_eq!( ParatensorModule::get_subnetwork_n(netuid) - 1 , uid );
		}
		// Build the graph with 10 items 
		// each with 1 stake and self weights.
		let n: usize = 10;
		let netuid: u16 = 0;
		ParatensorModule::set_max_allowed_uids( netuid, n as u16 ); 
		for i in 0..10 {
			add_node(
				netuid,
				i as u64,
				i as u64,
				i as u16,
				1
			)
		}
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), 10 );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		for i in 0..10 {
			assert_ok!(ParatensorModule::set_weights(Origin::signed(i), netuid, vec![ i as u16 ], vec![ u16::MAX ]));
			// ParatensorModule::set_weights_for_testing( netuid, i as u16, vec![ ( i as u16, u16::MAX )]); // doesn't set update status
			// ParatensorModule::set_bonds_for_testing( netuid, uid, vec![ ( i as u16, u16::MAX )]); // rather, bonds are calculated in epoch
		}
		// Run the epoch.
		ParatensorModule::epoch( 0, 1_000_000_000, true );
		// Check return values.
		for i in 0..n {
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(i as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, i as u16 ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, i as u16 ), 0 );
		}
	});
}

#[test]
// Test an epoch on a graph with two items.
fn test_4096_graph() {
	new_test_ext().execute_with(|| {
		let n: u16 = 4096;
		let validators: u16 = 200;
		let servers = n - validators;
		let netuid: u16 = 0;
        println!( "test_{n:?}_graph ({validators:?} validators)" );
		ParatensorModule::set_max_allowed_uids( netuid, n );
		for uid in 0..n {
			let stake: u128 = if uid < validators { 1 } else { 0 };
			ParatensorModule::add_balance_to_coldkey_account( &(uid as u64), stake );
			ParatensorModule::set_stake_for_testing( &(uid as u64), stake as u64 );
			ParatensorModule::add_subnetwork_account( netuid, uid, &(uid as u64) );
		   	ParatensorModule::increment_subnetwork_n( netuid );
		}
		assert_eq!( ParatensorModule::get_subnetwork_n(netuid), n );
		run_to_block( 1 ); // run to next block to ensure weights are set on nodes after their registration block
		for uid in 0..validators { // validators
			assert_ok!(ParatensorModule::set_weights(Origin::signed(uid as u64), netuid, (validators..n).collect(), vec![ u16::MAX / n; servers as usize ]));
		}
		for uid in validators..n { // servers
			assert_ok!(ParatensorModule::set_weights(Origin::signed(uid as u64), netuid, vec![ uid as u16 ], vec![ u16::MAX ])); // server self-weight
		}
		// Run the epoch.
		let start = Instant::now();
		ParatensorModule::epoch( netuid, 1_000_000_000, false );
		let duration = start.elapsed();
		println!("Time elapsed in epoch() is: {:?}", duration);

		for (uid, node) in vec![ (0, "validator"), (validators, "server") ] {
			println!( "\n{node}" );
			println!( "stake: {:?}", ParatensorModule::get_stake_for_hotkey( &(uid as u64) ));
			println!( "rank: {:?}", ParatensorModule::get_rank( netuid, uid ));
			println!( "trust: {:?}", ParatensorModule::get_trust( netuid, uid ));
			println!( "consensus: {:?}", ParatensorModule::get_consensus( netuid, uid ));
			println!( "incentive: {:?}", ParatensorModule::get_incentive( netuid, uid ));
			println!( "dividend: {:?}", ParatensorModule::get_dividend( netuid, uid ));
			println!( "emission: {:?}", ParatensorModule::get_emission( netuid, uid ));
		}
		for uid in 0..validators { // validators
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 1 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 327 );
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 2499995 );
		}
		for uid in validators..n { // servers
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(uid as u64) ), 0 );
			assert_eq!( ParatensorModule::get_rank( netuid, uid ), 16 );
			assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentive( netuid, uid ), 16 );
			assert_eq!( ParatensorModule::get_dividend( netuid, uid ), 0 );
			assert_eq!( ParatensorModule::get_emission( netuid, uid ), 128336 );
		}
	});
}