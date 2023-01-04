#![cfg_attr(not(feature = "std"), no_std)]
use pallet_paratensor::neuron_info::NeuronInfo as NeuronInfoStruct;
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs
sp_api::decl_runtime_apis! {
	pub trait NeuronInfoRuntimeApi {
		fn get_neurons(netuid: u16) -> Vec<NeuronInfoStruct>;
	}
}