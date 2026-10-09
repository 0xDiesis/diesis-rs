//! Checks addresses, chains, and the contracts table against canonical.json.

use alloy::primitives::Address;
use diesis::{addresses, chains, contracts, Chain, Contract, CANONICAL_JSON};
use serde_json::Value;

fn canonical() -> Value {
    serde_json::from_str(CANONICAL_JSON).expect("canonical.json parses")
}

fn canonical_address(name: &str) -> Address {
    canonical()["addresses"][name]
        .as_str()
        .unwrap_or_else(|| panic!("{name} missing from canonical.json"))
        .parse()
        .expect("valid address")
}

#[test]
fn schema_version_matches() {
    assert_eq!(canonical()["schemaVersion"], chains::SCHEMA_VERSION);
}

#[test]
fn every_canonical_address_has_a_constant() {
    let json = canonical();
    let entries = json["addresses"].as_object().unwrap();
    assert_eq!(addresses::ALL.len(), entries.len());
    for (name, address) in addresses::ALL {
        let expected = canonical_address(name);
        assert_eq!(address, expected, "{name}");
        assert_eq!(diesis::address(name), Some(expected));
    }
    for name in entries.keys() {
        assert!(diesis::address(name).is_some(), "{name} has no constant");
    }
    assert_eq!(diesis::address("NOT_AN_ADDRESS"), None);
}

#[test]
fn spot_checks_named_constants() {
    assert_eq!(
        addresses::DIESIS_STAKING,
        canonical_address("DIESIS_STAKING")
    );
    assert_eq!(addresses::WRAPPED_DS, canonical_address("WRAPPED_DS"));
    assert_eq!(addresses::PERMIT2, canonical_address("PERMIT2"));
}

#[test]
fn chains_match_canonical() {
    assert_eq!(chains::DIESIS.id, 1980);
    assert_eq!(chains::DIESIS.name, "Diesis");
    const { assert!(!chains::DIESIS.testnet) };
    assert_eq!(chains::DIESIS_TESTNET.id, 19803);
    assert_eq!(chains::DIESIS_TESTNET.name, "Diesis Testnet");
    const { assert!(chains::DIESIS_TESTNET.testnet) };

    let json = canonical();
    let entries = json["chains"].as_object().unwrap();
    assert_eq!(chains::ALL.len(), entries.len());
    for chain in chains::ALL {
        let entry = &entries[chain.key];
        assert_eq!(entry["id"], chain.id);
        assert_eq!(entry["name"], chain.name);
        assert_eq!(entry["rpcUrl"], chain.rpc_url);
        assert_eq!(entry["explorerUrl"], chain.explorer_url);
        assert_eq!(entry["testnet"], chain.testnet);
        let currency = &entry["nativeCurrency"];
        assert_eq!(currency["name"], chain.native_currency.name);
        assert_eq!(currency["symbol"], chain.native_currency.symbol);
        assert_eq!(currency["decimals"], chain.native_currency.decimals);
        assert_eq!(chain.native_currency.symbol, "DS");
        assert_eq!(chain.native_currency.decimals, 18);
        assert_eq!(Chain::by_id(chain.id), Some(&chain));
    }
    assert_eq!(Chain::by_id(1), None);
}

/// The TypeScript SDK's `diesisContracts`: key, contract name, address constant.
const EXPECTED_CONTRACTS: [(&str, &str, &str); 28] = [
    ("markets", "IDiesisMarkets", "DIESIS_MARKETS"),
    ("spotBook", "IDiesisSpotBook", "DIESIS_SPOT_BOOK"),
    ("perpsBook", "IDiesisPerpsBook", "DIESIS_PERPS_BOOK"),
    ("margin", "IDiesisMargin", "DIESIS_MARGIN"),
    ("settlement", "IDiesisSettlement", "DIESIS_SETTLEMENT"),
    (
        "settlementRouter",
        "DiesisSettlementRouter",
        "DIESIS_SETTLEMENT_ROUTER",
    ),
    ("conductors", "IDiesisConductors", "DIESIS_CONDUCTORS"),
    (
        "erc20Factory",
        "IDiesisErc20Factory",
        "DIESIS_ERC20_FACTORY",
    ),
    ("perpDeploy", "IDiesisPerpDeploy", "DIESIS_PERP_DEPLOY"),
    (
        "operatorBond",
        "IDiesisOperatorBond",
        "DIESIS_OPERATOR_BOND",
    ),
    (
        "bundleEscrow",
        "IDiesisBundleEscrow",
        "DIESIS_BUNDLE_ESCROW",
    ),
    ("staking", "DiesisStaking", "DIESIS_STAKING"),
    ("position", "IDiesisPosition", "DIESIS_POSITION"),
    ("liquidStakedDS", "ILiquidStakedDS", "LIQUID_STAKED_DS"),
    ("wrappedDS", "IWrappedDS", "WRAPPED_DS"),
    ("patron", "DiesisPatron", "DIESIS_PATRON"),
    ("config", "DiesisConfig", "DIESIS_CONFIG"),
    ("coreVault", "IDiesisCoreVault", "DIESIS_CORE_VAULT"),
    (
        "issuanceAuction",
        "IDiesisIssuanceAuction",
        "DIESIS_ISSUANCE_AUCTION",
    ),
    ("buybackBurn", "IDiesisBuybackBurn", "DIESIS_BUYBACK_BURN"),
    ("shieldedPool", "DiesisShieldedPool", "SHIELDED_POOL"),
    ("privacyPools", "DiesisPrivacyPools", "PRIVACY_POOLS"),
    (
        "nameRegistry",
        "IDiesisNameRegistry",
        "DIESIS_NAME_REGISTRY",
    ),
    (
        "baseRegistrar",
        "DiesisBaseRegistrar",
        "DIESIS_BASE_REGISTRAR",
    ),
    (
        "publicResolver",
        "DiesisPublicResolver",
        "DIESIS_PUBLIC_RESOLVER",
    ),
    (
        "reverseRegistrar",
        "IDiesisReverseRegistrar",
        "DIESIS_REVERSE_REGISTRAR",
    ),
    (
        "nameVerifier",
        "IDiesisNameVerifier",
        "DIESIS_NAME_VERIFIER",
    ),
    ("namePolicy", "IDiesisNamePolicy", "DIESIS_NAME_POLICY"),
];

#[test]
fn contracts_table_has_every_fixed_address_contract() {
    assert_eq!(contracts::ALL.len(), 28);
    for (contract, (key, name, address_name)) in contracts::ALL.iter().zip(EXPECTED_CONTRACTS) {
        assert_eq!(contract.key, key);
        assert_eq!(contract.name, name);
        assert_eq!(contract.address, canonical_address(address_name), "{key}");
        assert_eq!(Contract::by_key(key), Some(contract));
        assert_eq!(Contract::by_name(name), Some(contract));
        assert_eq!(Contract::by_address(contract.address), Some(contract));
    }
    assert_eq!(contracts::STAKING.address, addresses::DIESIS_STAKING);
    assert_eq!(contracts::SPOT_BOOK.key, "spotBook");
    assert_eq!(Contract::by_key("validatorShare"), None);
    assert_eq!(contracts::PUBLIC_CONTRACT_NAMES.len(), 29);
    assert!(contracts::PUBLIC_CONTRACT_NAMES.contains(&"IValidatorShare"));
}
