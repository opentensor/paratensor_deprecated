#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
use frame_support::inherent::Vec;
use pallet_paratensor::neuron_info::NeuronInfo as NeuronInfoStruct;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs
sp_api::decl_runtime_apis! {
	pub trait NeuronInfoApi {
        // TODO (Cameron): fix return type
		fn get_neurons(netuid: u16) -> Vec<NeuronInfoStruct>;
	}
}