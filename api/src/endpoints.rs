// Copyright 2016 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// pub struct HashID(pub [u8; 32]);
//
// impl FromStr for HashId {
//   type Err = ;
//
//   fn from_str(s: &str) -> Result<HashId, > {
//   }
// }

use std::sync::Arc;
use std::thread;

use core::core::Output;
use core::core::hash::Hash;
use chain::{self, Tip};
use rest::*;
use secp::pedersen::Commitment;
use util;

/// ApiEndpoint implementation for the blockchain. Exposes the current chain
/// state as a simple JSON object.
#[derive(Clone)]
pub struct ChainApi {
	/// data store access
	chain_store: Arc<chain::ChainStore>,
}

impl ApiEndpoint for ChainApi {
	type ID = String;
	type T = Tip;
	type OP_IN = ();
	type OP_OUT = ();

	fn operations(&self) -> Vec<Operation> {
		vec![Operation::Get]
	}

	fn get(&self, id: String) -> ApiResult<Tip> {
		self.chain_store.head().map_err(|e| Error::Internal(e.to_string()))
	}
}

/// ApiEndpoint implementation for outputs that have been included in the chain.
#[derive(Clone)]
pub struct OutputApi {
	/// data store access
	chain_store: Arc<chain::ChainStore>,
}

impl ApiEndpoint for OutputApi {
	type ID = String;
	type T = Output;
	type OP_IN = ();
	type OP_OUT = ();

	fn operations(&self) -> Vec<Operation> {
		vec![Operation::Get]
	}

	fn get(&self, id: String) -> ApiResult<Output> {
		debug!("GET output {}", id);
		let c = util::from_hex(id.clone()).map_err(|e| Error::Argument(format!("Not a valid commitment: {}", id)))?;
		self.chain_store
			.get_output_by_commit(&Commitment::from_vec(c))
			.map_err(|e| Error::Internal(e.to_string()))
	}
}

/// Start all server REST APIs. Just register all of them on a ApiServer
/// instance and runs the corresponding HTTP server.
pub fn start_rest_apis(addr: String, chain_store: Arc<chain::ChainStore>) {

	thread::spawn(move || {
		let mut apis = ApiServer::new("/v1".to_string());
		apis.register_endpoint("/chain".to_string(),
		                       ChainApi { chain_store: chain_store.clone() });
		apis.register_endpoint("/chain/output".to_string(),
		                       OutputApi { chain_store: chain_store.clone() });
		apis.start(&addr[..]).unwrap_or_else(|e| {
			error!("Failed to start API HTTP server: {}.", e);
		});
	});
}
