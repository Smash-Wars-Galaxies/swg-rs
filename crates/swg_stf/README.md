<div align="center">

# swg_stf

[<img alt="github" src="https://img.shields.io/badge/github-Smash--Wars--Galaxies/swg--rs-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/Smash-Wars-Galaxies/swg-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/swg_stf.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/swg_stf)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-swg_stf_-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/swg_stf)

</div>

## About

This library implements reading from and writing to STF files used by Star Wars Galaxies. 

Current tooling around this, such as [Sytner's Iff Editor](https://modthegalaxy.com/index.php?threads/about-sie.370/), 
mainly focus on allowing extracting, editing and combining in a user friendly way. However, there are no easy ways 
to build them into a content distribution pipeline.

This library, as well as others in this repository aim to provide building blocks and tools to simplify the data 
pipeline for editing and updating files required by servers and clients of the game.

## Usage

Add the following to your `Cargo.toml` using the [format](#formats) you want
to use:

```toml
[dependencies]
swg_stf = { version = "0.1.0" }
```

## MSRV

Our current Minimum Supported Rust Version is **1.73**.

## License

`swg_stf` is distributed under the terms of the GNU Affero General Public License (Version 3.0)

See [LICENSE](../LICENSE) for details.