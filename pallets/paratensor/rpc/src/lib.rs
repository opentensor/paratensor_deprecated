use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::Block as BlockT,
};
use std::sync::Arc;

pub use paratensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi;
use pallet_paratensor::neuron_info::NeuronInfo as NeuronInfoStruct;

#[rpc]
pub trait ParatensorCustomApi<BlockHash> {
	#[rpc(name = "neuronInfo_getNeurons")]
	fn get_neurons(&self, netuid: u16, at: Option<BlockHash>) -> Result<Vec<NeuronInfoStruct>>;
	#[rpc(name = "neuronInfo_getNeuron")]
	fn get_neuron(&self, netuid: u16, uid: u16, at: Option<BlockHash>) -> Result<Option<NeuronInfoStruct>>;
}

pub struct ParatensorCustom<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> ParatensorCustom<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

impl<C, Block> ParatensorCustomApi<<Block as BlockT>::Hash> for ParatensorCustom<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: NeuronInfoRuntimeApi<Block>,
	{ 
	fn get_neurons(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<NeuronInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_neurons(&at, netuid).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get neurons info.".into(),
			data: Some(e.to_string().into()),
		})
	}

	fn get_neuron(&self, netuid: u16, uid: u16, at: Option<<Block as BlockT>::Hash>) -> Result<Option<NeuronInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.get_neuron(&at, netuid, uid).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get neuron info.".into(),
			data: Some(e.to_string().into()),
		})
	}
}