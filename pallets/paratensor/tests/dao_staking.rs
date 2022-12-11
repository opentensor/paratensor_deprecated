mod mock;
use mock::*;
use frame_system::{Config};
use frame_support::sp_runtime::DispatchError;
use frame_support::weights::{GetDispatchInfo, DispatchInfo, DispatchClass, Pays};

/***********************************************************
	dao staking tests.
************************************************************/
#[test]
fn test_add_dao_stake_dispatch_info_ok() {
	new_test_ext().execute_with(|| {
		let ammount_staked = 5000;
        let call = Call::ParatensorModule( ParatensorCall::add_dao_stake{ ammount_staked } );
		assert_eq!(call.get_dispatch_info(), DispatchInfo {
			weight: 0,
			class: DispatchClass::Normal,
			pays_fee: Pays::No
		});
	});
}

#[test]
fn test_add_dao_stake_err_signature() {
	new_test_ext().execute_with(|| {
		let amount = 20000 ; // Not used
		let result = ParatensorModule::add_dao_stake(<<Test as Config>::Origin>::none(), amount);
		assert_eq!(result, DispatchError::BadOrigin.into());
	});
}


