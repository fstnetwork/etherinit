#!/usr/bin/env bash

export RUST_BACKTRACE=1

export NETWORK_NAME="f8k-ethereum"
export GENESIS_BLOCK_GAS_LIMIT="0x6422c84"

export CONSENSUS_ENGINE="aura"
export AURA_CONSENSUS_PARAMETERS='{"blockPeriod":5,"blockReward":"6000000000000000000"}'

export SEALER_MNEMONIC_PHRASE="rose rocket invest real refuse margin festival danger anger border idle brown"
export SEALER_INTRINSIC_BALANCE="777864512312937"
export MINER_COUNT=3

export ACCOUNT_STATES_FILE=$(mktemp)
cat >$ACCOUNT_STATES_FILE <<EOF
{
  "0x0000000000000000000000000000000000000094": {
    "balance": "1232343891813242341",
    "nonce": 3,
    "constructor": "0011223344556677889900aabbccddeeff"
  },
  "0x0000000000000000000000000000000000e3a949": {
    "balance": "",
    "nonce": "29"
  },
  "0x0000000000000000000000000000000000000949": {
    "balance": "0x",
    "nonce": "29"
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

cargo run generate-chainspec
