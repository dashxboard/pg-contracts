# Contribution Guidelines

Contributions to this project are welcome and appreciated. This document provides guidelines for the contribution process.

## Code of Conduct

This project repository and everyone participating in it is governed by the [**Code of Conduct**](/CODE_OF_CONDUCT.md). All contributors are expected to adhere to this code.

## Your First Contribution

### Setting Up the Project

To contribute, the project must be set up on a local machine.

1.  **Fork** the `pg-contracts` repository to a personal GitHub account.
2.  **Clone** the forked repository to a local machine:

```bash
git clone https://github.com/dashxboard/pg-contracts.git
cd pg-contracts
```

### Development Environment Requirements

To contribute to this Rust/Soroban smart contract project, ensure your development environment is properly configured:

#### Prerequisites

1. **Rust Toolchain**: Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **WASM Target**: Add the WebAssembly compilation target:

```bash
rustup target add wasm32v1-none
```

3. **Stellar CLI**: Install the Stellar CLI for contract deployment and testing:

```bash
cargo install --locked stellar-cli
```

4. **Identity Configuration**: Create a named identity for signing transactions and contract deployments:

```bash
stellar keys generate --global deployer --network testnet --fund
```

This creates an identity called "deployer" that is:

- Stored globally in `~/.config/stellar/identity/deployer.toml`
- Configured for Stellar testnet.
- Automatically funded via Friendbot for testing purposes.

You can verify the identity was created by checking the public key:

```bash
stellar keys address deployer
```

#### Building the Project

Compile all governance contracts using the recommended Stellar CLI commands:

```bash
# Build all contracts (recommended method)
stellar contract build

# Or build manually with cargo (for Rust v1.85.0+)
cargo build --target wasm32v1-none --release
```

#### Optimizing for Deployment

For production deployments, optimize the WASM files to minimize size:

```bash
# Optimize built contracts (only needed for deployment)
stellar contract optimize --wasm target/wasm32v1-none/release/<contract_name>.wasm
```

Replace `<contract_name>` with the specific contract you want to optimize (e.g., `token_weighted_vote_contract`, `token_gated_vote_contract`, etc.).

#### Running Tests

Execute the comprehensive test suite before submitting contributions:

```bash
# Run all tests
cargo test
```

## Reporting Issues and Requesting Features

Bugs are reported and new features are requested by opening an [**Issue**](https://github.com/dashxboard/pg-contracts/issues) using the appropriate template.

For feature development, the scope of the change should be considered:

- **Major Features** An issue must be opened to propose and discuss the design before coding begins.

- **Small Features & Bug Fixes** A [**Pull Request**](https://github.com/dashxboard/pg-contracts/pulls) can be submitted directly.

## Submission Guidelines

### Submitting an Issue

Before creating an issue, please search existing issues to see if a similar one has already been reported.

### Submitting a Pull Request (PR)

Follow these steps when submitting a Pull Request:

1. Search the project's pull requests to see if a similar one already exists.

2. Ensure an issue exists that documents the problem or feature. This provides context for the changes.

3. Fork the repository and create a new git branch from `main`. A descriptive branch name is recommended:

```bash
# Creates the new branch and switches to it
git checkout -b <branch-name> main
```

4. After making changes, they should be staged and committed with a clear message following the specified [format](#commit-message-format):

```bash
# Stage all changes
git add .

# Commit the changes
git commit -m "<type>(<scope>): <subject>"
```

5. Push the branch to the fork on GitHub:

```bash
git push origin <branch-name>
```

6. Open a pull request from the forked repository to the `pg-contracts:main` branch. The PR must be linked to the relevant issue.

**Note on Collaboration:** Pull requests that do not follow these guidelines may be closed with a request for corrections to ensure a smooth process.

## Testing Requirements

All contributions must include appropriate tests and pass the existing test suite:

### Test Coverage Requirements

- **New Features**: Must include comprehensive unit tests covering all functionality.
- **Bug Fixes**: Must include regression tests that verify the fix.
- **Contract Changes**: Must maintain 100% test pass rate across all governance contracts.

### Running Tests Before Submission

Before submitting a pull request, ensure all tests pass:

```bash
# Verify all contracts compile successfully
cargo check

# Run the complete test suite
cargo test

# Run tests with output for debugging
cargo test -- --nocapture
```

### Test Quality Standards

- Tests should be **deterministic** and **repeatable**.
- Each test should focus on a **single functionality**.
- Test names should clearly **describe the scenario** being tested.
- Use descriptive assertions with clear error messages.

### After a PR is Merged

Once a PR is merged, the related branches can be safely cleaned up.

1. Delete the remote branch on GitHub:

```bash
git push origin --delete <branch-name>
```

2. Switch back to the `main` branch:

```bash
git checkout main
```

3. Delete the local branch:

```bash
# Refuses if unmerged
git branch -d <branch-name>
```

```bash
# Force delete (if needed)
git branch -D <branch-name>
```

4. Update the local `main` branch with the latest changes from the upstream repository:

```bash
git pull upstream main
```

### Commit Message Format

The [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) specification is followed to maintain a clear and readable project repository history.

Each commit message consists of a **header**, an optional **body**, and an optional **footer**.

```
<type>(<scope>): <subject>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

- The `header` is required and must follow the format `<type>(<scope>): <subject>`.
- If present, the `body` should be used to explain the _what_ and _why_ of the changes, not the _how_.
- The `footer` should contain any breaking change information or reference issues that this commit closes.

### Reverting Commits

To revert a previous commit, the commit message must start with `revert:`, followed by the header of the commit being reverted. The body should explain the reason for the reversion.
