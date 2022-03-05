# nft-maker
The main function of this smart contract is to mint NFTs for players, and the minting fee is paid by the platform.

## Installation

### Rust

Go [here](https://www.rust-lang.org/tools/install) to install Rust.

### Solana

Go [here](https://docs.solana.com/cli/install-solana-cli-tools) to install Solana.

### Yarn

Go [here](https://yarnpkg.com/getting-started/install) to install Yarn.

### Anchor

npm i @project-serum/anchor-cli@0.20.1 -g

## Configuration

set devnet cluster

> solana config set --url=devnet

download mpl_token_metadata program:

> solana program dump -u m metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ./mpl_token_metadata.so

switch to localhost cluster

> solana config set --url=localhost

starting a local validator with load mpl_token_metadata.so program at destination address

> solana-test-validator --bpf-program metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s ./mpl_token_metadata.so -r

## Build

> git clone https://github.com/gameyoo/nft-maker.git
>
> cd nft-maker
> 
> yarn install
>
> anchor build
>
> anchor deploy
>
> anchor test

