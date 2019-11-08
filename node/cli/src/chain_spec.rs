// Copyright 2018 Commonwealth Labs, Inc.
// This file is part of Edgeware.

// Edgeware is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Edgeware is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Edgeware.  If not, see <http://www.gnu.org/licenses/>

use primitives::{Pair, Public};
use edgeware_primitives::{BlockNumber, AccountId, AuraId, Balance};
use im_online::ed25519::{AuthorityId as ImOnlineId};
use edgeware_runtime::{
	GrandpaConfig, BalancesConfig, ContractsConfig, ElectionsConfig, DemocracyConfig, CouncilConfig,
	AuraConfig, IndicesConfig, SessionConfig, StakingConfig, SudoConfig, TreasuryRewardConfig,
	SystemConfig, ImOnlineConfig, WASM_BINARY, Perbill, SessionKeys, StakerStatus, AuthorityDiscoveryConfig,
};
use edgeware_runtime::constants::{time::DAYS, currency::DOLLARS, currency::MILLICENTS};
use edgeware_runtime::{IdentityConfig, SignalingConfig};
use hex::FromHex;
pub use edgeware_runtime::GenesisConfig;
use primitives::crypto::UncheckedInto;
use serde::{Deserialize, Serialize};
use serde_json::{Result};
use std::fs::File;
use std::io::Read;
use substrate_service;
use substrate_telemetry::TelemetryEndpoints;
use grandpa::AuthorityId as GrandpaId;
use crate::mainnet_fixtures::*;
use crate::testnet_fixtures::*;
use sr_primitives::{
	traits::{One},
};
use core::convert::TryInto;

const DEFAULT_PROTOCOL_ID: &str = "edg";

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: client::ForkBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = substrate_service::ChainSpec<
	GenesisConfig,
	Extensions,
>;

#[derive(Serialize, Deserialize)]
struct Allocation {
    balances: Vec<(String, String)>,
    vesting: Vec<(String, u32, u32, String)>,
    validators: Vec<(String, String, String, String)>,
}

/// Development config (single validator Alice)
pub fn development_chainspec() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		|| { development_genesis_config(
			vec![
				get_authority_keys_from_seed("Alice"),
			],
			get_authority_keys_from_seed("Alice").0,
		) },
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		None)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_chainspec() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		|| { development_genesis_config(
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			get_authority_keys_from_seed("Alice").0,
		) },
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		None)
}

/// Edgeware public network config (mainnet and public testnets)
pub fn edgeware_chainspec(is_testnet: bool) -> ChainSpec {
	let boot_nodes = if is_testnet { get_testnet_bootnodes() } else { get_mainnet_bootnodes() };
	let data = r#"
		{
			"tokenDecimals": 18,
			"tokenSymbol": "EDG"
		}"#;
	let properties = serde_json::from_str(data).unwrap();
	ChainSpec::from_genesis(
		if is_testnet { "Edgeware Testnet" } else { "Edgeware" },
		if is_testnet { "edgeware-testnet" } else { "edgeware" },
		if is_testnet { edgeware_testnet_config } else { edgeware_mainnet_config },
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		properties,
	)
}

/// Helper function to create GenesisConfig for testing
pub fn development_genesis_config(
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId, ImOnlineId)>,
	root_key: AccountId,
) -> GenesisConfig {
	let extra_accounts: Vec<AccountId> = vec![
		get_account_id_from_seed::<AccountId>("Aaron"),
		get_account_id_from_seed::<AccountId>("Abigail"),
		get_account_id_from_seed::<AccountId>("Adam"),
		get_account_id_from_seed::<AccountId>("Alan"),
		get_account_id_from_seed::<AccountId>("Albert"),
		get_account_id_from_seed::<AccountId>("Alex"),
	];
	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const ENDOWMENT_STAKED: Balance = 9_000_000 * DOLLARS;

	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: extra_accounts.iter().cloned().map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), ENDOWMENT)))
				.chain(initial_authorities.iter().map(|x| (x.1.clone(), ENDOWMENT)))
				.collect(),
			vesting: vec![],
		}),
		indices: Some(IndicesConfig {
			ids: extra_accounts.iter().cloned()
				.chain(initial_authorities.iter().map(|x| x.0.clone()))
				.chain(initial_authorities.iter().map(|x| x.1.clone()))
				.collect::<Vec<_>>(),
		}),
		session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x|
				(x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone()))
			).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			validator_count: 7,
			minimum_validator_count: 4,
			stakers: initial_authorities.iter().map(|x| (x.0.clone(), x.1.clone(), ENDOWMENT_STAKED, StakerStatus::Validator)).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		democracy: Some(DemocracyConfig::default()),
		collective_Instance1: Some(CouncilConfig {
			members: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			phantom: Default::default(),
		}),
		elections: Some(ElectionsConfig {
			members: initial_authorities.iter().map(|x| (x.1.clone(), 1000000)).collect(),
			desired_seats: (initial_authorities.len() as u32) + 2,
			presentation_duration: (1 * DAYS).try_into().unwrap(),
			term_duration: (28 * DAYS).try_into().unwrap(),
		}),
		contracts: Some(ContractsConfig {
			current_schedule: Default::default(),
			gas_price: 1 * MILLICENTS,
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		aura: Some(AuraConfig {
			authorities: vec![],
		}),
		grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		authority_discovery: Some(AuthorityDiscoveryConfig{
			keys: vec![],
		}),
		identity: Some(IdentityConfig {
			verifiers: vec![get_account_id_from_seed::<AccountId>("Alice")],
			expiration_length: (1 * DAYS).try_into().unwrap(),
			registration_bond: 1 * DOLLARS,
		}),
		signaling: Some(SignalingConfig {
			voting_length: (3 * DAYS).try_into().unwrap(),
			proposal_creation_bond: 100 * DOLLARS,
		}),
		treasury_reward: Some(TreasuryRewardConfig {
			current_payout: 158 * DOLLARS,
			minting_interval: One::one(),
		}),
	}
}

/// Helper function to create GenesisConfig for public networks
pub fn edgeware_testnet_config() -> GenesisConfig {
	// validators
	let cw_authorities: Vec<(AccountId, AccountId, AuraId, Balance, GrandpaId, ImOnlineId)> = get_cw_testnet_validators();

	let session_keys = cw_authorities.iter().map(|x| (x.0.clone(), session_keys(x.2.clone(), x.4.clone(), x.5.clone())))
		.collect::<Vec<_>>();

	// balances
	let cw_allocation: Vec<(AccountId, Balance)> = get_commonwealth_allocation();
	let lockdrop_allocation = get_lockdrop_participants_allocation(true).unwrap();
	let lockdrop_balances = lockdrop_allocation.0;
	let lockdrop_vesting = lockdrop_allocation.1;

	// other configuration items
	let root_key = get_testnet_root_key();
	let identity_verifiers = get_testnet_identity_verifiers();
	let election_members = get_testnet_election_members();

	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: cw_allocation.iter().map(|x| (x.0.clone(), x.1.clone()))
				.chain(cw_authorities.iter().map(|x| (x.0.clone(), x.3.clone()))) // stash accounts
				.chain(cw_authorities.iter().map(|x| (x.1.clone(), CONTROLLER_ENDOWMENT))) // controller accounts
				.chain(lockdrop_balances.iter().map(|x| (x.0.clone(), x.1.clone()))) // lockdrop accounts
				.collect(),
			vesting: lockdrop_vesting,
		}),
		indices: Some(IndicesConfig {
			ids: cw_allocation.iter().map(|x| x.0.clone())
				.chain(cw_authorities.iter().map(|x| x.0.clone()))
				.chain(cw_authorities.iter().map(|x| x.1.clone()))
				.chain(lockdrop_balances.iter().map(|x| x.0.clone()))
				.collect::<Vec<_>>(),
		}),
		session: Some(SessionConfig {
			keys: session_keys,
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			validator_count: 60,
			minimum_validator_count: 0,
			stakers: cw_authorities.iter().map(|x| (
				x.0.clone(),
				x.1.clone(),
				// Ensure stakers have some non-bonded balance
				x.3.clone() - 10000000000000000000,
				StakerStatus::Validator
			)).collect(),
			invulnerables: vec![],
			slash_reward_fraction: Perbill::from_percent(0),
			.. Default::default()
		}),
		democracy: Some(DemocracyConfig::default()),
		collective_Instance1: Some(CouncilConfig {
			members: election_members.iter().map(|x| x.clone()).collect(),
			phantom: Default::default(),
		}),
		elections: Some(ElectionsConfig {
			members: election_members.iter().map(|x| (x.clone(), 6 * 28 * DAYS)).collect(),
			desired_seats: 4,
			presentation_duration: (1 * DAYS).try_into().unwrap(),
			term_duration: (30 * DAYS).try_into().unwrap(),
		}),
		contracts: Some(ContractsConfig {
			current_schedule: Default::default(),
			gas_price: 1 * MILLICENTS,
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		aura: Some(AuraConfig {
			authorities: vec![],
		}),
		grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		authority_discovery: Some(AuthorityDiscoveryConfig{
			keys: vec![],
		}),
		identity: Some(IdentityConfig {
			verifiers: identity_verifiers,
			expiration_length: (7 * DAYS).try_into().unwrap(),
			registration_bond: 1 * DOLLARS,
		}),
		signaling: Some(SignalingConfig {
			voting_length: (14 * DAYS).try_into().unwrap(),
			proposal_creation_bond: 100 * DOLLARS,
		}),
		treasury_reward: Some(TreasuryRewardConfig {
			current_payout: 95 * DOLLARS,
			minting_interval: One::one(),
		}),
	}
}

/// Helper function to create GenesisConfig for public networks
pub fn edgeware_mainnet_config() -> GenesisConfig {
	// validators
	let cw_authorities: Vec<(AccountId, AccountId, AuraId, Balance, GrandpaId, ImOnlineId)> = get_cw_mainnet_validators();
	let initial_lockdrop_authorities: Vec<(AccountId, AccountId, AuraId, Balance, GrandpaId, ImOnlineId)> = get_lockdrop_mainnet_validators();
	let session_keys = cw_authorities.iter().map(|x| (x.0.clone(), session_keys(x.2.clone(), x.4.clone(), x.5.clone())))
		.chain(initial_lockdrop_authorities.iter().map(|x| (x.0.clone(), session_keys(x.2.clone(), x.4.clone(), x.5.clone()))))
		.collect::<Vec<_>>();

	// balances
	let cw_allocation: Vec<(AccountId, Balance)> = get_commonwealth_allocation();
	let lockdrop_allocation = get_lockdrop_participants_allocation(false).unwrap();
	let lockdrop_balances = lockdrop_allocation.0;
	let lockdrop_vesting = lockdrop_allocation.1;

	// other configuration items
	let root_key = get_mainnet_root_key();
	let identity_verifiers = get_mainnet_identity_verifiers();
	let election_members = get_mainnet_election_members();

	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: cw_allocation.iter().map(|x| (x.0.clone(), x.1.clone()))
				.chain(cw_authorities.iter().map(|x| (x.0.clone(), x.3.clone()))) // stash accounts
				.chain(cw_authorities.iter().map(|x| (x.1.clone(), CONTROLLER_ENDOWMENT))) // controller accounts
				.chain(lockdrop_balances.iter().map(|x| (x.0.clone(), x.1.clone()))) // lockdrop accounts
				.collect(),
			vesting: lockdrop_vesting,
		}),
		indices: Some(IndicesConfig {
			ids: cw_allocation.iter().map(|x| x.0.clone())
				.chain(cw_authorities.iter().map(|x| x.0.clone()))
				.chain(cw_authorities.iter().map(|x| x.1.clone()))
				.chain(lockdrop_balances.iter().map(|x| x.0.clone()))
				.collect::<Vec<_>>(),
		}),
		session: Some(SessionConfig {
			keys: session_keys,
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			validator_count: 60,
			minimum_validator_count: 0,
			stakers: cw_authorities.iter().map(|x| (
				x.0.clone(),
				x.1.clone(),
				// Ensure stakers have some non-bonded balance
				x.3.clone() - 10000000000000000000,
				StakerStatus::Validator
			)).chain(initial_lockdrop_authorities.iter().map(|x| (
				x.0.clone(),
				x.1.clone(),
				x.3.clone() - 10000000000000000000,
				StakerStatus::Validator
			))).collect(),
			invulnerables: vec![],
			slash_reward_fraction: Perbill::from_percent(0),
			.. Default::default()
		}),
		democracy: Some(DemocracyConfig::default()),
		collective_Instance1: Some(CouncilConfig {
			members: election_members.iter().map(|x| x.clone()).collect(),
			phantom: Default::default(),
		}),
		elections: Some(ElectionsConfig {
			members: election_members.iter().map(|x| (x.clone(), 6 * 28 * DAYS)).collect(),
			desired_seats: 11,
			presentation_duration: (3 * DAYS).try_into().unwrap(),
			term_duration: (180 * DAYS).try_into().unwrap(),
		}),
		contracts: Some(ContractsConfig {
			current_schedule: Default::default(),
			gas_price: 1 * MILLICENTS,
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		aura: Some(AuraConfig {
			authorities: vec![],
		}),
		grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		authority_discovery: Some(AuthorityDiscoveryConfig{
			keys: vec![],
		}),
		identity: Some(IdentityConfig {
			verifiers: identity_verifiers,
			expiration_length: (7 * DAYS).try_into().unwrap(),
			registration_bond: 1 * DOLLARS,
		}),
		signaling: Some(SignalingConfig {
			voting_length: (14 * DAYS).try_into().unwrap(),
			proposal_creation_bond: 100 * DOLLARS,
		}),
		treasury_reward: Some(TreasuryRewardConfig {
			current_payout: 95 * DOLLARS,
			minting_interval: One::one(),
		}),
	}
}

/// Helper function for session keys
fn session_keys(aura: AuraId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
	SessionKeys { aura, grandpa, im_online }
}

// Give each account the amount specified in the lockdrop allocation,
// or 1000 EDG if we are generating a testnet spec
const TESTNET_DEFAULT_BALANCE: Balance = 1000000000000000000000;
fn get_lockdrop_participants_allocation(equalize_balances: bool) -> Result<(
	Vec<(AccountId, Balance)>,
	Vec<(AccountId, BlockNumber, BlockNumber, Balance)>,
)> {
	let mut file = File::open("lockdrop_allocations.json").unwrap();
	let mut data = String::new();
	file.read_to_string(&mut data).unwrap();

	let json: Allocation = serde_json::from_str(&data)?;
	let balances_json = json.balances;
	let vesting_json = json.vesting;

	let balances: Vec<(AccountId, Balance)> = balances_json.into_iter().map(|e| {
		return (
			<[u8; 32]>::from_hex(e.0).unwrap().unchecked_into(),
			if equalize_balances { TESTNET_DEFAULT_BALANCE } else { e.1.to_string().parse::<Balance>().unwrap() },
		);
	}).collect();
	let vesting: Vec<(AccountId, BlockNumber, BlockNumber, Balance)> = vesting_json.into_iter().map(|e| {
		return (
			<[u8; 32]>::from_hex(e.0).unwrap().unchecked_into(),
			e.1.to_string().parse::<BlockNumber>().unwrap(),
			e.2.to_string().parse::<BlockNumber>().unwrap(),
			if equalize_balances { TESTNET_DEFAULT_BALANCE } else { e.1.to_string().parse::<Balance>().unwrap() },
		);
	}).collect();
	Ok((balances, vesting))
}


/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}


/// Helper function to generate stash, controller and session key from seed
fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuraId, GrandpaId, ImOnlineId) {
	(
		get_account_id_from_seed::<AccountId>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<AccountId>(seed),
		get_from_seed::<AuraId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
	)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}
