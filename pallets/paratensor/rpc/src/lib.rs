use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use paratensor_custom_rpc_runtime_api::NeuronInfoApi as NeuronInfoRuntimeApi;
use pallet_paratensor::neuron_info::NeuronInfo as NeuronInfoStruct;

#[rpc]
pub trait NeuronInfoApi<BlockHash> {
    // TODO (Cameron): fix return type
	#[rpc(name = "neuronInfo_getNeurons")]
	fn get_neurons(&self, netuid: u16, at: Option<BlockHash>) -> Result<Vec<NeuronInfoStruct>>;
}

pub struct NeuronInfo<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> NeuronInfo<C, M> {
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

impl<C, Block> NeuronInfoApi<<Block as BlockT>::Hash> for NeuronInfo<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: NeuronInfoRuntimeApi<Block>,
	{   // TODO (Cameron): fix return type
	fn get_neurons(&self, netuid: u16, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<NeuronInfoStruct>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_neurons(&at, netuid);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}