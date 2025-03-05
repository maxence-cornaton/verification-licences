# A tool to check memberships for the French Society of Unicycling (CNM)

[![Bin](https://github.com/maxence-cornaton/verification-licences/actions/workflows/bin.yml/badge.svg)](https://github.com/maxence-cornaton/verification-licences/actions/workflows/bin.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
---

## Introduction

At the CNM, we've had issues checking manually every and each person who'd like to take in an event. This involved a long and tedious process. This app strives to simplify the process.

## Getting started
The following tools are required:
- [Rust](https://www.rust-lang.org/): version 1.85+ (supporting Rust Edition 2024)
- `wasm32-unknown-unknown` toolchain: install using `rustup target add wasm32-unknown-unknown`
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/): required to run WASM tests
- [Docker](https://www.docker.com/): used to package the app in an easy-to-share image

Once everything's installed, you can try and compile the app:
1. On Windows, run `.\build-wasm.bat` to build the WASM lib. If you use another OS, please adapt the script - it should not be too hard.
2. Build and run the app in demo mode with `cargo run --features demo`. 
3. If that's the first time you run the app, you'll have to populate the memberships. You can do so with cURL or any other tool: `curl --request GET \
  --url http://127.0.0.1:8000/api/memberships`.
4. Once the app is started and populated, go to http://127.0.0.1:8000/check-memberships. You should be able to check memberships.

If you'd like to run the app against real data, run the app with the following arguments:
```shell
cargo run -- -l=<login> -p=<password>
```
You'll need Fileo credentials to do so.

## File Structure

The project is structured as follows:
- [dto](https://github.com/maxence-cornaton/verification-licences/tree/main/dto): a library with all shared DTOs.
- [public](https://github.com/maxence-cornaton/verification-licences/tree/main/public): the client-side assets (images, templates).
- [src](https://github.com/maxence-cornaton/verification-licences/tree/main/src): the main app location. Includes the server, the data retrieval logic and the validation logic.
- [wasm](https://github.com/maxence-cornaton/verification-licences/tree/main/wasm): the client-side code, written in Rust and compiled into WASM.

Besides, throughout your journey into the app, you'll encounter some generated folders:
- _data/_: location for the downloaded memberships. This acts as the database.
- _demo_data/_: similarly to `data`, this is the location for the demo memberships. This is populated when running in demo mode.
- _public/static/pkg/_: this is the location for the generated WASM and JS libs.