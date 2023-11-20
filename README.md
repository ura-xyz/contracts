# Ura

Ura is a community-governed DEX on Terra that is focused on providing the best rates and an unparalleled user experience. It takes inspiration from Solidly and Velodrome's flywheel tokenomics and is the first to introduce the ve(3,3) DEX in the Cosmos.

## Contracts

This repository contains the source code for the CosmWasm smart contracts of Ura.

| Name                                                     | Description                                  |
| -------------------------------------------------------- | -------------------------------------------- |
| [`factory`](./contracts/factory)                           | Pair factory and controller                  |
| [`minter`](./contracts/minter)                             | URA token minter (controls emissions)        |
| [`native_coin_registry`](./contracts/native_coin_registry) | Registry of native coin decimals             |
| [`pair`](./contracts/pair)                                 | Pair with x*y=k curve                        |
| [`pair_stable`](./contracts/pair_stable)                   | Pair with stableswap invariant curve         |
| [`router`](./contracts/router)                             | Multi-hop trade router                       |
| [`token`](./contracts/token)                               | CW20 (ERC20 equivalent) token implementation |

## Developing

You will need Rust 1.64.0+ with `wasm32-unknown-unknown` target installed.

### Testing

To run all test:

```sh
cargo test
```

### Building

To build an optimized build ready to be deployed to the chain, run the following from the repository root:

```sh
./build.sh
```

The optimized contracts will be generated in the root `/artifacts` directory.

## Credits

- [Terraswap](https://github.com/terraswap/terraswap) / [Astroport](https://github.com/astroport-fi/astroport-core): from which a large part of the `factory`, `native_coin_registry`, `pair`, `pair_stable`, and `router` contracts are adapted from
- [White Whale](https://github.com/White-Whale-Defi-Platform/white-whale-core): from which part of the `minter` contract is adapted from
- [Solidly](https://solidly.com/swap/) / [Velodrome](https://velodrome.finance/): for inspiring the mechanics and foundation of Ura
