use crate::{mock::*};
#[cfg(feature = "no_std")]
use ndarray::{ndarray::Array1, ndarray::Array2, ndarray::arr1};
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
		ParatensorModule::set_weights_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]);
		ParatensorModule::set_bonds_for_testing( netuid, uid, vec![ ( 0, u16::MAX )]);
		ParatensorModule::epoch( 0, 0, true );
		assert_eq!( ParatensorModule::get_stake_for_hotkey( &hotkey ), stake_amount );
		assert_eq!( ParatensorModule::get_ranks( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_trust( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_consensus( netuid, uid ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
		assert_eq!( ParatensorModule::get_incentives( netuid, uid ), 0 );
		assert_eq!( ParatensorModule::get_dividends( netuid, uid ), 0 );
	});
}

#[test]
// Test an epoch on a graph with two items.
fn test_2_graph() {
	new_test_ext().execute_with(|| {
        println!( "test_2_graph:" );
		// Function for adding a nodes to the graph.
		pub fn add_node( 
				netuid: u16,
				coldkey: u64, 
				hotkey:u64, 
				uid: u16, 
				stake_amount: u64,
				weights: Vec<(u16, u16)>,
				bonds: Vec<(u16, u16)>,
			){
			ParatensorModule::add_balance_to_coldkey_account( &coldkey, stake_amount as u128 );
			ParatensorModule::set_stake_for_testing( &hotkey, stake_amount );
			ParatensorModule::add_subnetwork_account( netuid, uid, &hotkey );
		   	ParatensorModule::increment_subnetwork_n( netuid );
		   	ParatensorModule::set_weights_for_testing( netuid, uid, weights);
			ParatensorModule::set_bonds_for_testing( netuid, uid, bonds);
		}
		// Build the graph with two items 
		// each with 1 stake and self weights.
		let n: usize = 2;
		let netuid: u16 = 0;
		for i in 0..n {
			add_node(
				netuid,
				i as u64,
				i as u64,
				i as u16,
				1,
				vec![ ( i as u16, u16::MAX )],
				vec![ ( i as u16, u16::MAX )]
			)
		}
		// Run the epoch.
		ParatensorModule::epoch( 0, 0, true );
		// Check return values.
		for i in 0..n {
			assert_eq!( ParatensorModule::get_stake_for_hotkey( &(i as u64) ), 1 );
			assert_eq!( ParatensorModule::get_ranks( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_trust( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_consensus( netuid, i as u16 ), 438 ); // Note C = 0.0066928507 = (0.0066928507*65_535) = floor( 438.6159706245 )
			assert_eq!( ParatensorModule::get_incentives( netuid, i as u16 ), 0 );
			assert_eq!( ParatensorModule::get_dividends( netuid, i as u16 ), 0 );	
		}
	});
}