//! Checks the abi-typegen bindings.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use alloy::json_abi::JsonAbi;
use alloy::primitives::{address, Address, U256};
use alloy::sol_types::{SolCall, SolEvent};
use diesis::bindings::*;
use diesis::contracts::{self, PUBLIC_CONTRACT_NAMES};

/// Each public contract: Solidity name, embedded JSON ABI, and the ABI that
/// alloy's `sol!` macro built from the generated Solidity.
macro_rules! public_bindings {
    ($($module:ident :: $contract:ident => $abi:ident),* $(,)?) => {
        fn public_bindings() -> Vec<(&'static str, &'static str, JsonAbi)> {
            vec![$((stringify!($contract), $module::$abi, $contract::abi::contract())),*]
        }
    };
}

public_bindings! {
    i_diesis_markets::IDiesisMarkets => I_DIESIS_MARKETS_ABI,
    i_diesis_spot_book::IDiesisSpotBook => I_DIESIS_SPOT_BOOK_ABI,
    i_diesis_perps_book::IDiesisPerpsBook => I_DIESIS_PERPS_BOOK_ABI,
    i_diesis_margin::IDiesisMargin => I_DIESIS_MARGIN_ABI,
    i_diesis_settlement::IDiesisSettlement => I_DIESIS_SETTLEMENT_ABI,
    diesis_settlement_router::DiesisSettlementRouter => DIESIS_SETTLEMENT_ROUTER_ABI,
    i_diesis_conductors::IDiesisConductors => I_DIESIS_CONDUCTORS_ABI,
    i_diesis_erc20_factory::IDiesisErc20Factory => I_DIESIS_ERC20_FACTORY_ABI,
    i_diesis_perp_deploy::IDiesisPerpDeploy => I_DIESIS_PERP_DEPLOY_ABI,
    i_diesis_operator_bond::IDiesisOperatorBond => I_DIESIS_OPERATOR_BOND_ABI,
    i_diesis_bundle_escrow::IDiesisBundleEscrow => I_DIESIS_BUNDLE_ESCROW_ABI,
    diesis_staking::DiesisStaking => DIESIS_STAKING_ABI,
    i_diesis_position::IDiesisPosition => I_DIESIS_POSITION_ABI,
    i_validator_share::IValidatorShare => I_VALIDATOR_SHARE_ABI,
    i_liquid_staked_ds::ILiquidStakedDS => I_LIQUID_STAKED_DS_ABI,
    i_wrapped_ds::IWrappedDS => I_WRAPPED_DS_ABI,
    diesis_patron::DiesisPatron => DIESIS_PATRON_ABI,
    diesis_config::DiesisConfig => DIESIS_CONFIG_ABI,
    i_diesis_core_vault::IDiesisCoreVault => I_DIESIS_CORE_VAULT_ABI,
    i_diesis_issuance_auction::IDiesisIssuanceAuction => I_DIESIS_ISSUANCE_AUCTION_ABI,
    i_diesis_buyback_burn::IDiesisBuybackBurn => I_DIESIS_BUYBACK_BURN_ABI,
    diesis_shielded_pool::DiesisShieldedPool => DIESIS_SHIELDED_POOL_ABI,
    diesis_privacy_pools::DiesisPrivacyPools => DIESIS_PRIVACY_POOLS_ABI,
    i_diesis_name_registry::IDiesisNameRegistry => I_DIESIS_NAME_REGISTRY_ABI,
    diesis_base_registrar::DiesisBaseRegistrar => DIESIS_BASE_REGISTRAR_ABI,
    diesis_public_resolver::DiesisPublicResolver => DIESIS_PUBLIC_RESOLVER_ABI,
    i_diesis_reverse_registrar::IDiesisReverseRegistrar => I_DIESIS_REVERSE_REGISTRAR_ABI,
    i_diesis_name_verifier::IDiesisNameVerifier => I_DIESIS_NAME_VERIFIER_ABI,
    i_diesis_name_policy::IDiesisNamePolicy => I_DIESIS_NAME_POLICY_ABI,
}

fn signatures(abi: &JsonAbi) -> BTreeSet<String> {
    let functions = abi
        .functions()
        .map(|f| format!("function {}", f.signature()));
    let events = abi.events().map(|e| format!("event {}", e.signature()));
    let errors = abi.errors().map(|e| format!("error {}", e.signature()));
    functions.chain(events).chain(errors).collect()
}

fn contracts_dir() -> PathBuf {
    match std::env::var_os("DIESIS_CONTRACTS_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../diesis-core/diesis/contracts"),
    }
}

/// The embedded JSON ABI of every public contract parses with alloy's
/// `JsonAbi` and has the same functions, events, and errors as the `sol!`
/// binding.
#[test]
fn embedded_abis_match_sol_bindings() {
    let bindings = public_bindings();
    let names: Vec<&str> = bindings.iter().map(|(name, _, _)| *name).collect();
    assert_eq!(names, PUBLIC_CONTRACT_NAMES);
    for (name, json, sol_abi) in &bindings {
        let abi: JsonAbi = serde_json::from_str(json)
            .unwrap_or_else(|err| panic!("{name} ABI does not parse: {err}"));
        assert!(abi.functions().next().is_some(), "{name} has no functions");
        assert_eq!(signatures(&abi), signatures(sol_abi), "{name}");
    }
}

/// The contracts table carries the same ABI as the binding module.
#[test]
fn contracts_table_abis_come_from_bindings() {
    let bindings = public_bindings();
    for contract in contracts::ALL {
        let (_, json, _) = bindings
            .iter()
            .find(|(name, _, _)| *name == contract.name)
            .unwrap_or_else(|| panic!("{} has no binding", contract.name));
        assert!(std::ptr::eq(contract.abi, *json), "{}", contract.key);
    }
}

/// The embedded ABIs match the compiled artifacts in the contracts checkout.
/// Skipped when the artifacts are not available.
#[test]
fn embedded_abis_match_artifacts() {
    let out = contracts_dir().join("out");
    if !out.is_dir() {
        eprintln!(
            "skipping: no contract artifacts at {} (set DIESIS_CONTRACTS_DIR)",
            out.display()
        );
        return;
    }
    for (name, json, _) in public_bindings() {
        let artifact = out.join(format!("{name}.sol/{name}.json"));
        let artifact: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&artifact).unwrap()).unwrap();
        let expected: JsonAbi = serde_json::from_value(artifact["abi"].clone()).unwrap();
        let embedded: JsonAbi = serde_json::from_str(json).unwrap();
        assert_eq!(
            embedded, expected,
            "{name} is stale, run scripts/generate.py"
        );
    }
}

#[test]
fn calls_encode_with_canonical_selectors() {
    let abi: JsonAbi = serde_json::from_str(diesis_staking::DIESIS_STAKING_ABI).unwrap();
    let stake = &abi.function("stake").unwrap()[0];
    assert_eq!(DiesisStaking::stakeCall::SIGNATURE, "stake(uint256)");
    assert_eq!(DiesisStaking::stakeCall::SELECTOR, stake.selector().0);

    let call = DiesisStaking::stakeCall {
        toValidatorId: U256::from(7),
    };
    let data = call.abi_encode();
    assert_eq!(&data[..4], stake.selector().as_slice());
    let back = DiesisStaking::stakeCall::abi_decode(&data).unwrap();
    assert_eq!(back.toValidatorId, U256::from(7));

    let returns =
        DiesisStaking::stakeCall::abi_decode_returns(&U256::from(42).to_be_bytes::<32>()).unwrap();
    assert_eq!(returns, U256::from(42));
}

#[test]
fn bundle_escrow_uses_initial_wire_selectors() {
    let abi: JsonAbi =
        serde_json::from_str(i_diesis_bundle_escrow::I_DIESIS_BUNDLE_ESCROW_ABI).unwrap();
    let expected = [
        (
            "reserveBundle",
            [0x79, 0x3b, 0xae, 0xf2],
            IDiesisBundleEscrow::reserveBundleCall::SELECTOR,
        ),
        (
            "finalizeBundle",
            [0x1e, 0x73, 0xb2, 0xe6],
            IDiesisBundleEscrow::finalizeBundleCall::SELECTOR,
        ),
        (
            "cancelBundle",
            [0xd3, 0x07, 0xc0, 0xa3],
            IDiesisBundleEscrow::cancelBundleCall::SELECTOR,
        ),
        (
            "reclaimExpiredBundle",
            [0x3d, 0xc7, 0x27, 0xf7],
            IDiesisBundleEscrow::reclaimExpiredBundleCall::SELECTOR,
        ),
    ];
    for (name, selector, generated) in expected {
        assert_eq!(selector, generated, "{name} generated selector");
        assert_eq!(
            selector,
            abi.function(name).unwrap()[0].selector().0,
            "{name} ABI selector"
        );
    }
    for old in [
        "reserveBundleV2",
        "finalizeBundleV2",
        "cancelBundleV2",
        "reclaimExpiredBundleV2",
    ] {
        assert!(
            !abi.functions.contains_key(old),
            "obsolete {old} selector remains"
        );
    }
}

#[test]
fn events_encode_and_serialize() {
    let delegator: Address = address!("0x0000000000000000000000000000000000000001");
    let event = DiesisStaking::Staked {
        delegator,
        toValidatorId: U256::from(1),
        tokenId: U256::from(42),
        amount: U256::from(10).pow(U256::from(18)),
    };
    let log = event.encode_log_data();
    assert_eq!(log.topics().len(), 4);
    assert_eq!(log.topics()[0], DiesisStaking::Staked::SIGNATURE_HASH);
    let decoded = DiesisStaking::Staked::decode_log_data(&log).unwrap();
    assert_eq!(decoded, event);

    let text = serde_json::to_string(&event).unwrap();
    let back: DiesisStaking::Staked = serde_json::from_str(&text).unwrap();
    assert_eq!(back, event);
}

/// The type names the README mentions exist.
#[test]
fn readme_type_names_exist() {
    let best_ask = IDiesisSpotBook::getBestAskReturn {
        priceTicks: U256::from(1),
        amountLots: U256::from(2),
    };
    assert_eq!(best_ask.amountLots, U256::from(2));
    assert_eq!(
        DiesisStaking::safeTransferFrom_0Call::SIGNATURE,
        "safeTransferFrom(address,address,uint256)"
    );
    assert!(!DiesisStaking::DiesisStakingErrors::SELECTORS.is_empty());
}
