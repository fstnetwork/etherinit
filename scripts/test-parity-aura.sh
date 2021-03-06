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
export PARITY_TX_QUEUE_SIZE=4096
export PARITY_TX_QUEUE_MEM_LIMIT=7
export PARITY_TX_QUEUE_PER_SENDER=12

export ETHEREUM_PROGRAM="parity"
export RUNNING_MODE="development"

export CONSENSUS_ENGINE="Aura"
export AURA_CONSENSUS_PARAMETERS='{"blockPeriod": 5}'

export GENESIS_BLOCK_GAS_LIMIT="0x11E1A300"

export BOOTNODE_SERVICE_HOST="localhost"
export BOOTNODE_SERVICE_PORT=9292

export ACCOUNT_STATES_FILE=$(mktemp)
export SEALER_INTRINSIC_BALANCE="646464646464646464"

export RUST_BACKTRACE=1

trap "pkill etherinit; pkill parity; exit 0" INT

cat >$ACCOUNT_STATES_FILE <<EOF
{
  "0x0000000000000000000000000000000000000094": {
    "balance": "1232343891813242341",
    "nonce": 3
  },
  "0x0053f97dc01ce07602b208f844b35e8484acf69f": {
    "balance": "8908974907345139",
    "nonce": "29"
  },
  "0x0172bf37b2ff1bc5ff140634d9981011f54ae6aa": {
    "balance": "0x123ba2",
    "nonce": "0x29"
  },
  "0x0098071f663ff4eaf0cdbbed17d253f2f93adcb1": {
    "balance": 9231111
  },
  "0x0131261ee085b48ff47c67729c2f2dd5a95caa2f": {
    "balance": 3239930
  },
  "0x014b554d5cdac30cf1af1783759cd9085b198042": {
    "balance": 1111923
  },
  "0x0172bf37b2ff1bc5ff140634d9981011f54ae6aa": {
    "balance": "3234829231111"
  },
  "0x01b62af9a06c5dac2e62dd1f88ee5325324ab910": {
    "balance": 12899231111
  },
  "0x020351d19ebbfae3a3aa3a016f3b4f516e0c2b69": {
    "balance": 3929231111
  }
}
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
  export IPC_PATH="$CONFIG_ROOT/miner-$i.ipc"
  export BASE_PATH="$CONFIG_ROOT/base"

  echo $i $CONFIG_ROOT

  rm -v -rf $CONFIG_ROOT/first-run-lock
  rm -v -rf $CONFIG_ROOT/parity-config

  RUST_LOG=info ./target/debug/etherinit run-ethereum init
  parity --config=$CONFIG_ROOT/config.toml &
  IPC_PATH="$CONFIG_ROOT/miner-$i.ipc" ./target/debug/etherinit run-network-keeper &
done

for ((i = 0; i < $TRANSACTOR_COUNT; i++)); do
  export NODE_ROLE=Transactor
  export TRANSACTOR_INDEX=$i

  export P2P_NETWORK_SERVICE_PORT=$(($i + 40303))
  export HTTP_JSON_RPC_PORT=$(($i + 28545))
  export WEBSOCKET_JSON_RPC_PORT=$(($i + 38546))

  export CHAIN_DATA_ROOT="$ROOT_PREFIX/transactor-$TRANSACTOR_INDEX/chain-data"
  export CONFIG_ROOT="$ROOT_PREFIX/transactor-$TRANSACTOR_INDEX"
  export BASE_PATH="$CONFIG_ROOT/base"

  echo $i $CONFIG_ROOT $HOME

  rm -v -rf $CONFIG_ROOT/first-run-lock
  rm -v -rf $CONFIG_ROOT/parity-config

  export PARITY_LOGGING=""
  RUST_LOG=info ./target/debug/etherinit run-ethereum full &
done

for ((i = 0; i < $SYNCER_COUNT; i++)); do
  export NODE_ROLE=Syncer
  export SYNCER_INDEX=$i

  export P2P_NETWORK_SERVICE_PORT=$(($i + 50303))
  export HTTP_JSON_RPC_PORT=$(($i + 48545))
  export WEBSOCKET_JSON_RPC_PORT=$(($i + 58546))

  export CHAIN_DATA_ROOT="$ROOT_PREFIX/syncer-$SYNCER_INDEX/chain-data"
  export CONFIG_ROOT="$ROOT_PREFIX/syncer-$SYNCER_INDEX"
  export BASE_PATH="$CONFIG_ROOT/base"

  echo $i $CONFIG_ROOT $HOME

  rm -v -rf $CONFIG_ROOT/first-run-lock
  rm -v -rf $CONFIG_ROOT/parity-config

  export PARITY_LOGGING=""
  RUST_LOG=info ./target/debug/etherinit run-ethereum full &
done

while true; do
  sleep 1
done
