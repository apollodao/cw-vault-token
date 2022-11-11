# Cosmwasm Vault Token

## Description

An abstraction for different ways of implementing a vault token.
This crate defines a set of traits that define the behavior of a vault
token. Two implementations are provided, one for an Osmosis native denom
minted through the TokenFactory module and one for Cw4626 tokenized vaults.
See the cosmwasm-vault-standard crate for more information about tokenized
vaults.
