use cumulus_primitives_core::ParaId;
use bittensor_parachain::{AccountId, AuraId, Signature, EXISTENTIAL_DEPOSIT};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_core::crypto::{Ss58Codec,Ss58AddressFormatRegistry};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<bittensor_parachain::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> bittensor_parachain::SessionKeys {
	bittensor_parachain::SessionKeys { aura: keys }
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
	root_key: AccountId,
) -> bittensor_parachain::GenesisConfig {
	bittensor_parachain::GenesisConfig {
		system: bittensor_parachain::SystemConfig {
			code: bittensor_parachain::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: bittensor_parachain::BalancesConfig {
			//balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
			balances: vec![ 
				(Ss58Codec::from_ss58check("5EqeLybpo51F5tdn4JrDEG9sWacgZ4ZgHaHUGU86sNvPQjE9").unwrap(),6058535716465)
				],
		},
		sudo: bittensor_parachain::SudoConfig { key: Some(root_key) },
		parachain_info: bittensor_parachain::ParachainInfoConfig { parachain_id: id },
		collator_selection: bittensor_parachain::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: bittensor_parachain::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: bittensor_parachain::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}

fn finney_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
	root_key: AccountId,
) -> bittensor_parachain::GenesisConfig {
	bittensor_parachain::GenesisConfig {
		system: bittensor_parachain::SystemConfig {
			code: bittensor_parachain::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: bittensor_parachain::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		sudo: bittensor_parachain::SudoConfig { key: Some(root_key) },
		parachain_info: bittensor_parachain::ParachainInfoConfig { parachain_id: id },
		collator_selection: bittensor_parachain::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: bittensor_parachain::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		polkadot_xcm: bittensor_parachain::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}

pub fn kusama_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TAO".into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Bittensor Kusama",
		// ID
		"Bittensor",
		ChainType::Live,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					// Collator 1
					(
						Ss58Codec::from_ss58check("5DRijXqKWJBR4wLdT9vAJaXHMbATnECnYX4HG48UV9pL9m8z").unwrap(),
						Ss58Codec::from_ss58check("5DRijXqKWJBR4wLdT9vAJaXHMbATnECnYX4HG48UV9pL9m8z").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5FTs5Hawze8EXSQN7qx5cs9SboExFk7hmLA7rWZu95xptDAF").unwrap(),
						Ss58Codec::from_ss58check("5FTs5Hawze8EXSQN7qx5cs9SboExFk7hmLA7rWZu95xptDAF").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5DUwBWrha8Rht55SyFLNXzK79UTBhNnut2q83hr1Q3ARWXih").unwrap(),
						Ss58Codec::from_ss58check("5DUwBWrha8Rht55SyFLNXzK79UTBhNnut2q83hr1Q3ARWXih").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5CkHa4m4Kc4thsDxaoRJUJRS9pZ7xo7omqMboUoGereHFCXN").unwrap(),
						Ss58Codec::from_ss58check("5CkHa4m4Kc4thsDxaoRJUJRS9pZ7xo7omqMboUoGereHFCXN").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5DsvShVtJp53Yhuc7hov2vUAVvQNQ7Bqz5eJc2PeWbEAUURB").unwrap(),
						Ss58Codec::from_ss58check("5DsvShVtJp53Yhuc7hov2vUAVvQNQ7Bqz5eJc2PeWbEAUURB").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5F46fcTzcMSm7e6Tu9W6DmEeLcA1wE6rxdN42fniWFZJRdWp").unwrap(),
						Ss58Codec::from_ss58check("5F46fcTzcMSm7e6Tu9W6DmEeLcA1wE6rxdN42fniWFZJRdWp").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("Fk3FnurqREFZ7CW7Vc44k6magDAvmn1oNmWEiG9gJPjxZMP").unwrap(),
						Ss58Codec::from_ss58check("Fk3FnurqREFZ7CW7Vc44k6magDAvmn1oNmWEiG9gJPjxZMP").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("HC4AUuwxHALGCcHo3JAo6XgLKA5nQhy4cjgGjLpFKhbpyzT").unwrap(),
						Ss58Codec::from_ss58check("HC4AUuwxHALGCcHo3JAo6XgLKA5nQhy4cjgGjLpFKhbpyzT").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("GvUoJw9CVqdmmRTkGcAfPyUbBWU8k7bQXHYUg66dJZLypYA").unwrap(),
						Ss58Codec::from_ss58check("GvUoJw9CVqdmmRTkGcAfPyUbBWU8k7bQXHYUg66dJZLypYA").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("GXk9MJ2xbcTrtwoLwShET6grTjW961b5ha8bwFSgukdtWLC").unwrap(),
						Ss58Codec::from_ss58check("GXk9MJ2xbcTrtwoLwShET6grTjW961b5ha8bwFSgukdtWLC").unwrap(),
					),
				],
				vec![
					Ss58Codec::from_ss58check("5DRijXqKWJBR4wLdT9vAJaXHMbATnECnYX4HG48UV9pL9m8z").unwrap(),
				],
				2245.into(),
				Ss58Codec::from_ss58check("5HfwmrEq1iUPWmTtfbm1XSsPiSrw9dMraosh7sQcNmb6frsJ").unwrap()
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("bittensor"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "kusama".into(), // You MUST set this to the correct network!
			para_id: 2245,
		},
	)
}

pub fn rococo_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TAO".into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Bittensor Rococo Testnet",
		// ID
		"bittensor_rococo_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					// Collator 1
					(
						Ss58Codec::from_ss58check("5DRijXqKWJBR4wLdT9vAJaXHMbATnECnYX4HG48UV9pL9m8z").unwrap(),
						Ss58Codec::from_ss58check("5DRijXqKWJBR4wLdT9vAJaXHMbATnECnYX4HG48UV9pL9m8z").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5FTs5Hawze8EXSQN7qx5cs9SboExFk7hmLA7rWZu95xptDAF").unwrap(),
						Ss58Codec::from_ss58check("5FTs5Hawze8EXSQN7qx5cs9SboExFk7hmLA7rWZu95xptDAF").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5DUwBWrha8Rht55SyFLNXzK79UTBhNnut2q83hr1Q3ARWXih").unwrap(),
						Ss58Codec::from_ss58check("5DUwBWrha8Rht55SyFLNXzK79UTBhNnut2q83hr1Q3ARWXih").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5CkHa4m4Kc4thsDxaoRJUJRS9pZ7xo7omqMboUoGereHFCXN").unwrap(),
						Ss58Codec::from_ss58check("5CkHa4m4Kc4thsDxaoRJUJRS9pZ7xo7omqMboUoGereHFCXN").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5DsvShVtJp53Yhuc7hov2vUAVvQNQ7Bqz5eJc2PeWbEAUURB").unwrap(),
						Ss58Codec::from_ss58check("5DsvShVtJp53Yhuc7hov2vUAVvQNQ7Bqz5eJc2PeWbEAUURB").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5F46fcTzcMSm7e6Tu9W6DmEeLcA1wE6rxdN42fniWFZJRdWp").unwrap(),
						Ss58Codec::from_ss58check("5F46fcTzcMSm7e6Tu9W6DmEeLcA1wE6rxdN42fniWFZJRdWp").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("Fk3FnurqREFZ7CW7Vc44k6magDAvmn1oNmWEiG9gJPjxZMP").unwrap(),
						Ss58Codec::from_ss58check("Fk3FnurqREFZ7CW7Vc44k6magDAvmn1oNmWEiG9gJPjxZMP").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("HC4AUuwxHALGCcHo3JAo6XgLKA5nQhy4cjgGjLpFKhbpyzT").unwrap(),
						Ss58Codec::from_ss58check("HC4AUuwxHALGCcHo3JAo6XgLKA5nQhy4cjgGjLpFKhbpyzT").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("GvUoJw9CVqdmmRTkGcAfPyUbBWU8k7bQXHYUg66dJZLypYA").unwrap(),
						Ss58Codec::from_ss58check("GvUoJw9CVqdmmRTkGcAfPyUbBWU8k7bQXHYUg66dJZLypYA").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("GXk9MJ2xbcTrtwoLwShET6grTjW961b5ha8bwFSgukdtWLC").unwrap(),
						Ss58Codec::from_ss58check("GXk9MJ2xbcTrtwoLwShET6grTjW961b5ha8bwFSgukdtWLC").unwrap(),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				2004.into(),
				Ss58Codec::from_ss58check("5EqeLybpo51F5tdn4JrDEG9sWacgZ4ZgHaHUGU86sNvPQjE9").unwrap()
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("bittensor_rococo_testnet"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
			para_id: 2004,
		},
	)
}

pub fn polkadot_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "TAO".into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("ss58Format".into(), 42.into());

	ChainSpec::from_genesis(
		// Name
		"Bittensor",
		// ID
		"bittensor",
		ChainType::Live,
		move || {
			finney_genesis(
				// initial collators.
				vec![
					// Collator 1
					(
						Ss58Codec::from_ss58check("5CyEApos33zYijdejJLKmtDe5G9ZHu8j5U5gAFLVtMbSH397").unwrap(),
						Ss58Codec::from_ss58check("5CyEApos33zYijdejJLKmtDe5G9ZHu8j5U5gAFLVtMbSH397").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5C4vTsttJhs1EtgX8ud6zKD7Um7Nz7YSEGfMYHKYUnfRo7sB").unwrap(),
						Ss58Codec::from_ss58check("5C4vTsttJhs1EtgX8ud6zKD7Um7Nz7YSEGfMYHKYUnfRo7sB").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5EfN5FdDkTLpvwWuFM5Nmv16u26badc7L7ZrcHFnUfjoiEaL").unwrap(),
						Ss58Codec::from_ss58check("5EfN5FdDkTLpvwWuFM5Nmv16u26badc7L7ZrcHFnUfjoiEaL").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5CLmhtrxcR5Nr9JRg7fPCFeoKYpMKxwFv6zFJtvqcsRMEJYk").unwrap(),
						Ss58Codec::from_ss58check("5CLmhtrxcR5Nr9JRg7fPCFeoKYpMKxwFv6zFJtvqcsRMEJYk").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5DhFds8pMw4zxf2Xo3QVyz6PJLnNr2mUR2ddhhVy4dMB6VeH").unwrap(),
						Ss58Codec::from_ss58check("5DhFds8pMw4zxf2Xo3QVyz6PJLnNr2mUR2ddhhVy4dMB6VeH").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5FqXjUf4x4erYGZqEKFne4oRp1v59n5wTQLoYqsR4GQriThb").unwrap(),
						Ss58Codec::from_ss58check("5FqXjUf4x4erYGZqEKFne4oRp1v59n5wTQLoYqsR4GQriThb").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5HEdfuYU8QgDYMEPztjSNr19VjsG6E3TvSyE411GiYLn6tuQ").unwrap(),
						Ss58Codec::from_ss58check("5HEdfuYU8QgDYMEPztjSNr19VjsG6E3TvSyE411GiYLn6tuQ").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5FeBMdHzUSg92m7oC9m2Utard7R6frjzZCQi4tFLzvQfT7L6").unwrap(),
						Ss58Codec::from_ss58check("5FeBMdHzUSg92m7oC9m2Utard7R6frjzZCQi4tFLzvQfT7L6").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5G9SwYm7maYMuEYG2WpFtN97BmyG6JnYuqpCAyUyKteJpPDv").unwrap(),
						Ss58Codec::from_ss58check("5G9SwYm7maYMuEYG2WpFtN97BmyG6JnYuqpCAyUyKteJpPDv").unwrap(),
					),
					(
						Ss58Codec::from_ss58check("5G42S5TM8RcmRtfGMwEpKcMt5Mj74UD2EvMRrS9oihv6c23e").unwrap(),
						Ss58Codec::from_ss58check("5G42S5TM8RcmRtfGMwEpKcMt5Mj74UD2EvMRrS9oihv6c23e").unwrap(),
					),
				],
				vec![
					// Collator 1
					Ss58Codec::from_ss58check("5CyEApos33zYijdejJLKmtDe5G9ZHu8j5U5gAFLVtMbSH397").unwrap(),
										
				],
				2097.into(),
				Ss58Codec::from_ss58check("5GmxWYKFb6ecktezdcCpeU6xQSTXvpqiw3tZ7sKSn3NbJAEC").unwrap()
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("bittensor"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "polkadot".into(), // You MUST set this to the correct network!
			para_id: 2097,
		},
	)
}

