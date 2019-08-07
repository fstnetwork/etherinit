#!/usr/bin/env bash

# WARN: make sure you execute this script in root directory of project

export NETWORK_NAME="f8k-ethereum"
export ETHEREUM_PROGRAM="parity"
export BOOTNODE_SERVICE_HOST="localhost"
export BOOTNODE_SERVICE_PORT=9292
export IPC_PATH="http://127.0.0.1:8545/"

export RUST_BACKTRACE=1
export RUST_LOG=info

cargo run run-network-keeper
