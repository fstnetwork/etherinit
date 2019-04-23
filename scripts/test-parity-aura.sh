#!/usr/bin/env bash

# WARN: make sure you execute this script in root directory of project
cargo build

ROOT_PREFIX="/tmp/ethereum-launcher-test"

mkdir -p $ROOT_PREFIX

export NETWORK_NAME="f8k-ethereum"
export SEALER_MNEMONIC_PHRASE="rose rocket invest real refuse margin festival danger anger border idle brown"
export MINER_COUNT=3
export TRANSACTOR_COUNT=2
export SYNCER_COUNT=1
export MIN_GAS_LIMIT="0x7A1200"
export PARITY_GAS_FLOOR_TARGET="870000"
export PARITY_GAS_CAP="1870000"

export CONSENSUS_ENGINE="Aura"
export AURA_CONSENSUS_PARAMETERS='{"blockPeriod": 5}'

export GENESIS_BLOCK_GAS_LIMIT="0x11E1A300"

export BOOTNODE_SERVICE_HOST="localhost"
export BOOTNODE_SERVICE_PORT=9292

export ACCOUNT_BALANCES_FILE=$(mktemp)
export SEALER_INTRINSIC_BALANCE="646464646464646464"

export RUST_BACKTRACE=1

trap "pkill etherinit; pkill parity; exit 0" INT

cat >$ACCOUNT_BALANCES_FILE <<EOF
[{
    "address": "0x0000000000000000000000000000000000000004",
    "balance": "1232343891813242341"
}, {
    "address": "0x0053f97dc01ce07602b208f844b35e8484acf69f",
    "balance": "1231312312312312312"
}, {
    "address": "0x00726c0aa4673269a003b23dccd6e7f2b2229c86",
    "balance": "1367554854854848"
}, {
    "address": "0x0098071f663ff4eaf0cdbbed17d253f2f93adcb1",
    "balance": "1367554854854848"
}, {
    "address": "0x0131261ee085b48ff47c67729c2f2dd5a95caa2f",
    "balance": "1367554854854857"
}, {
    "address": "0x014b554d5cdac30cf1af1783759cd9085b198042",
    "balance": "1367554854854848"
}, {
    "address": "0x0172bf37b2ff1bc5ff140634d9981011f54ae6aa",
    "balance": "1367554854854848"
}, {
    "address": "0x01b62af9a06c5dac2e62dd1f88ee5325324ab910",
    "balance": "1367554854854895"
}, {
    "address": "0x020351d19ebbfae3a3aa3a016f3b4f516e0c2b69",
    "balance": "1367554854854870"
}]
EOF

BOOTNODE_SOCKET="0.0.0.0:${BOOTNODE_SERVICE_PORT}" ./target/debug/etherinit run-bootnode-server &

for ((i = 0; i < $MINER_COUNT; i++)); do
  export NODE_ROLE=Miner
  export MINER_INDEX=$i

  export P2P_NETWORK_SERVICE_PORT=$(($i + 30303))
  export HTTP_JSON_RPC_PORT=$(($i + 8545))
  export WEBSOCKET_JSON_RPC_PORT=$(($i + 18546))

  export CHAIN_DATA_ROOT="$ROOT_PREFIX/miner-$MINER_INDEX/chain-data"
  export CONFIG_ROOT="$ROOT_PREFIX/miner-$MINER_INDEX"
  export XDG_CONFIG_HOME=$CONFIG_ROOT/config
  export XDG_DATA_HOME=$CONFIG_ROOT/data
  export HOME=$CONFIG_ROOT

  echo $i $CONFIG_ROOT $HOME

  rm -v -rf $CONFIG_ROOT/first-run-lock
  rm -v -rf $CONFIG_ROOT/parity-config

  RUST_LOG=info ./target/debug/etherinit run-ethereum &
done

for ((i = 0; i < $TRANSACTOR_COUNT; i++)); do
  export NODE_ROLE=Transactor
  export TRANSACTOR_INDEX=$i

  export P2P_NETWORK_SERVICE_PORT=$(($i + 40303))
  export HTTP_JSON_RPC_PORT=$(($i + 28545))
  export WEBSOCKET_JSON_RPC_PORT=$(($i + 38546))

  export CHAIN_DATA_ROOT="$ROOT_PREFIX/transactor-$TRANSACTOR_INDEX/chain-data"
  export CONFIG_ROOT="$ROOT_PREFIX/transactor-$TRANSACTOR_INDEX"
  export XDG_CONFIG_HOME=$CONFIG_ROOT/config
  export XDG_DATA_HOME=$CONFIG_ROOT/data
  export HOME=$CONFIG_ROOT

  echo $i $CONFIG_ROOT $HOME

  rm -v -rf $CONFIG_ROOT/first-run-lock
  rm -v -rf $CONFIG_ROOT/parity-config

  export PARITY_LOGGING=""
  RUST_LOG=info ./target/debug/etherinit run-ethereum &
done

for ((i = 0; i < $SYNCER_COUNT; i++)); do
  export NODE_ROLE=Syncer
  export SYNCER_INDEX=$i

  export P2P_NETWORK_SERVICE_PORT=$(($i + 50303))
  export HTTP_JSON_RPC_PORT=$(($i + 48545))
  export WEBSOCKET_JSON_RPC_PORT=$(($i + 58546))

  export CHAIN_DATA_ROOT="$ROOT_PREFIX/syncer-$SYNCER_INDEX/chain-data"
  export CONFIG_ROOT="$ROOT_PREFIX/syncer-$SYNCER_INDEX"
  export XDG_CONFIG_HOME=$CONFIG_ROOT/config
  export XDG_DATA_HOME=$CONFIG_ROOT/data
  export HOME=$CONFIG_ROOT

  echo $i $CONFIG_ROOT $HOME

  rm -v -rf $CONFIG_ROOT/first-run-lock
  rm -v -rf $CONFIG_ROOT/parity-config

  export PARITY_LOGGING=""
  RUST_LOG=info ./target/debug/etherinit run-ethereum &
done

while true; do
  sleep 1
done
