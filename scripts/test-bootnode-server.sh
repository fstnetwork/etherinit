#!/usr/bin/env bash

export BOOTNODE_SOCKET="0.0.0.0:9292"

export NETWORK_NAME="f8k-ethereum"
export GENESIS_BLOCK_GAS_LIMIT="0x6422c40"

export CONSENSUS_ENGINE="aura"
export AURA_CONSENSUS_PARAMETERS='{"blockPeriod":5,"blockReward":"6000000000000000000"}'

export SEALER_MNEMONIC_PHRASE="rose rocket invest real refuse margin festival danger anger border idle chalk"
export MINER_COUNT=7

export RUST_BACKTRACE=1
RUST_LOG=info cargo run run-bootnode-server
