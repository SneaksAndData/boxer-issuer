# Contributing to this project

This document explains everything you need to know to contribute.
Before you start please read the [development guide](DEVELOPMENT.md).

## Writing Code

If you want to submit a PR, make sure that you've discussed the solution with
the maintainers beforehand. We want to avoid situations where you put a lot of
work into a fix that we can't merge! If there's no issue for what you're trying
to fix yet, make one _before_ you start working on the PR.

## Code Style

We use `cargo fmt` for Rust code and `go fmt` for Go code. Please make sure to format your code before submitting a PR.
The code should be written in idiomatic Rust/Go style. If you're unsure about something, feel free to ask in the PR.

The project is structured in a multi-repository setup. The boxer-issuer and boxer-validator-nginx repositories contain
the main
application code the boxer-core contains the shared functionality. The boxer-crd repository contains the CRD definitions
used by the Kubernetes backend.

Don't do the following:

- Calling `unwrap` or `expect` in Rust code (except the tests). Use proper error handling instead.
- Creating the `mod.rs` files in Rust code. Use the `foo.rs` file instead of `foo/mod.rs`.
- Using the unsafe code in Rust code in this project.

## Pull Requests

Make sure you are following the checklist in the PR template.

## Introducing Breaking Changes

### Tokens

If you are introducing a breaking change in the token structure, please follow these steps:

1. Add the new version in the version claim of the token.
2. Add support for both the old and new version in the **validator** code.
3. Ensure that the **issuer** code is able to issue both the old and new version of the token.

### APIs

If you are introducing a breaking change in the API, please follow these steps:

1. Add the new version in the API path.
2. If you are changing the `token` endpoint, ensure that you proposed the corresponding changes in the following
   repositories:
    - `terraform-provider-boxer`
    - api clients (if any)
3. If you are changing other APIs, ensure that you proposed the corresponding changes in the following repositories:
    - `terraform-provider-boxer`

### Terraform Provider

If you are introducing a breaking change in the Terraform provider, please follow the hashicorp guidelines
for [introducing breaking changes](https://developer.hashicorp.com/terraform/plugin/sdkv2/best-practices/deprecations).