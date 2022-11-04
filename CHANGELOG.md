# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Bug Fixes

- Remove unused traits
- Clarifying comment
- Remove cosmwasm_1_1 feature from cosmwasm-std import
- Delete cw20 implementation
- Remove unused transfer fn
- Remove unused is_native fn
- Update dependencies, add empty cargo workspace, comment an doc e… (#16)

### Features

- Add VaulToken trait
- Implement query_total_supply for Cw20
- Impl query_total_supply for cw4626
- Impl query_total_supply for OsmosisDenom
- Impl From<CwTokenError> for StdError
- Update total supply query for osmosis to using stargate
- Replace apollo-proto-rust w/ osmosis-std
- Use cosmwasm1_1 feature for supply query
- Switch to using burn_from for cw4626
- Merge Token and VaultToken traits

### Fix

- Fix mint, burn, add AssertReceived

### Miscellaneous Tasks

- Update deps

### Refactor

- Rename token.rs to traits.rs

## [0.4.0] - 2022-10-05

### Bug Fixes

- Add _ match on AssetInfo
- Take item param as borrowed
- Derive from error
- Remove self as argument to instantiate
- Remove unused struct
- Make impl functions pub
- Add cw_serde derive and remove sized trait bound

### Features

- Remove T generic from Instantiate trait
- Implement cw4626
- Impl Instantiate and Send for Cw20
- Make osmosis init not need reply

### Miscellaneous Tasks

- Update cw-asset
- Make deps caret
- Update deps to cw 1.1
- Tag apollo-proto-rust dep

## [0.1.3] - 2022-07-22

### Features

- Add set_admin_addr function to Instantiate trait

## [0.1.2] - 2022-07-22

### Features

- Add traits to OsmosisDenomInstantiator, remove unneeded imports

## [0.1.1] - 2022-07-21

### Features

- Impl From<OsmosisDenom> for AssetInfo
- Implement Token trait

## [0.1.0] - 2022-07-21

### Bug Fixes

- Rename CwAssetError to CwTokenError

### Miscellaneous Tasks

- Bump version of apollo-proto-rust dependency

