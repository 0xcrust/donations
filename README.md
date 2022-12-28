## Installations
- [Rust](https://www.rust-lang.org/tools/install)
- [Solana](https://docs.solana.com/cli/install-solana-cli-tools)
- [Yarn](https://yarnpkg.com/getting-started/install)

View the full steps [here.](https://book.anchor-lang.com/getting_started/installation.html)

## Build and Testing
Deploy the contract to the `localnet` by following these steps on your cli:

#### Config
- `solana-keygen new` to create a wallet keypair
- `solana config set --url localhost` to set your rpcUrl to localhost
#### Build and deployment
- Clone the repo and cd into /program
- Run `cargo build-bpf`
- Start up the validator with `solana-test-validator`
- Run the bash script with `chmod 755 deploy.sh` and then `./deploy.sh` to deploy your program.
#### Testing
- Navigate into the `/scripts` directory
- Run `yarn install` to install dependencies
- `yarn run test` to run the tests









