mod mock;
use frame_system::Config;
use frame_support::{assert_ok};
use mock::*;

#[test]
fn test_block_step_multi(){
    new_test_ext().execute_with(|| { 
        // Create default network.
        let netuid0: u16 = 0;
        let netuid1: u16 = 1;
        let netuid2: u16 = 2;
        let tempo0: u16 = 0; // Never runs.
        let tempo1: u16 = 1; // Runs every block.
        let tempo2: u16 = 3; // Runs every other block.

        add_network( netuid0, tempo0, 0 );
        add_network( netuid1, tempo1, 0 );
        add_network( netuid2, tempo2, 0 );

        // // Lets step a block. There if no emission because we have not set an emission vector.
        assert_eq!( ParatensorModule::get_pending_emission( netuid0 ), 0 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid1 ), 0 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid2 ), 0 );
        step_block(1);
        assert_eq!( ParatensorModule::get_pending_emission( netuid0 ), 0 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid1 ), 0 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid2 ), 0 );

        // Lets set the block emission for this network. It will get all the emission.
        let netuids: Vec<u16> = vec![ 0, 1, 2];
        let emission: Vec<u64> = vec![ 333_333_333, 333_333_333, 333_333_334  ];
        assert_ok!( ParatensorModule::sudo_set_emission_values(<<Test as Config>::Origin>::root(), netuids, emission) );

        // Run a forward block. All emission ends up in pending.
        assert_eq!( ParatensorModule::get_emission_value( netuid0 ), 333_333_333 );
        assert_eq!( ParatensorModule::get_emission_value( netuid1 ), 333_333_333 );
        assert_eq!( ParatensorModule::get_emission_value( netuid2 ), 333_333_334 );
        step_block(1);
        assert_eq!( ParatensorModule::get_pending_emission( netuid0 ), 333_333_333 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid1 ), 333_333_333 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid2 ), 333_333_334 );

        // Run two more blocks and emission accrues for all networks.
        step_block(1);
        assert_eq!( ParatensorModule::get_pending_emission( netuid0 ), 666_666_666 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid1 ), 666_666_666 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid2 ), 666_666_668 );

        step_block(1);
        assert_eq!( ParatensorModule::get_pending_emission( netuid0 ), 999_999_999 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid1 ), 999_999_999 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid2 ), 1_000_000_002 );

        // Create keys.
		let hotkey0: u64 = 0;
		let coldkey0: u64 = 0;

        // Register 1 neuron to each network starting emission.
        register_ok_neuron( netuid0, hotkey0, coldkey0, 39420842 );
    	register_ok_neuron( netuid1, hotkey0, coldkey0, 12412392 );
		register_ok_neuron( netuid2, hotkey0, coldkey0, 21813123 );

        // Run the block.
        step_block(1);
        assert_eq!( ParatensorModule::get_pending_emission( netuid0 ), 1_333_333_332 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid1 ), 1_333_333_332 );
        assert_eq!( ParatensorModule::get_pending_emission( netuid2 ), 1_333_333_336 );


    });
}



#[test]
fn test_nakamoto(){
    new_test_ext().execute_with(|| { 

        // Create nakamoto.
        let n: u16 = 10;
        let netuid: u16 = 0;
        let tempo: u16 = 100;
        add_network( netuid, tempo, 0 );
        let netuids: Vec<u16> = vec![ 0 ];
        let emission: Vec<u64> = vec![ 1_000_000_000 ];
        assert_ok!( ParatensorModule::sudo_set_emission_values(<<Test as Config>::Origin>::root(), netuids, emission) );

        // Increase network size to 4096
        ParatensorModule::set_max_allowed_uids( netuid, n );
        ParatensorModule::set_max_registrations_per_block( netuid, n * 2 );

        // Register neurons.
        for i in 0..n as u64 {
            log::trace!( "Register:\n{:?}\n", i );
            register_ok_neuron( netuid, i, i, i * 1_000_000_000 + i * 1_000_000 );
            assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, i as u16 ).unwrap(), i );
        }

        // Register the next batch to replace the older ones.
        for i in 0..n as u64 {
            log::trace!( "Register:\n{:?}\n", i );
            register_ok_neuron( netuid, i + n as u64, i + n as u64 , i * 2_200_100_500 + i * 2_000_000 + 124124 );
            assert_eq!( ParatensorModule::get_hotkey_for_net_and_uid( netuid, i as u16 ).unwrap(), i + n as u64 );
        }

    });
}

