#!/usr/bin/env bash

export RUST_BACKTRACE=1

export NETWORK_NAME="f8k-ethereum"
export GENESIS_BLOCK_GAS_LIMIT="0x6422c84"

export CONSENSUS_ENGINE="aura"
export AURA_CONSENSUS_PARAMETERS='{"blockPeriod":5,"blockReward":"6000000000000000000"}'

export SEALER_MNEMONIC_PHRASE="rose rocket invest real refuse margin festival danger anger border idle brown"
export SEALER_INTRINSIC_BALANCE="777864512312937"
export MINER_COUNT=3

export ACCOUNT_BALANCES_FILE=$(mktemp)
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

cargo run generate-chainspec
