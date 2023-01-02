#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use pallet_paratensor::NeuronMetadata as NeuronMetadataStruct;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_metadata.rs
sp_api::decl_runtime_apis! {
	pub trait NeuronMetadataApi {
        // TODO (Cameron): fix return type
		fn get_neurons(netuid: u16) -> NeuronMetadataStruct;
	}
}