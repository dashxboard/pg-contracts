# Token-Gated Vote Contract

This contract implements a "_one holder, one vote_" democratic governance model where every token holder receives equal voting weight. Token ownership above zero qualifies users to vote, with each holder getting exactly one vote.

Key features include token eligibility verification, duplicate vote prevention, time-bounded proposals, overflow-safe vote counting, and secure admin controls. The contract includes 19 comprehensive tests covering initialization, error handling, proposal management, voting mechanics, and edge cases.

## Overview

**Voting Process:**

1. **Token Verification:** Users must hold any amount > 0 of the governance token to participate.
2. **Weight Assignment:** Every qualified holder receives exactly one vote.
3. **Duplicate Prevention:** The contract enforces one vote per holder per proposal.
4. **Vote Aggregation:** Tallies accumulate with equal weight.
5. **Overflow Protection:** Uses saturating arithmetic to prevent vote count manipulation.

**Proposal Lifecycle:**

1. **Creation:** Admin creates proposals with time validation (5 to 15-day duration limits).
2. **Voting Period:** Token holders cast votes during the active time window.
3. **Vote Counting:** Each vote counts as one unit for all token holders.
4. **Resolution:** A simple majority determines the outcome.

## Getting Started

### Prerequisites

- **Rust & Soroban Environment**: Set up the environment for building, deploying, and interacting with Soroban contracts. Detailed instructions are available in the [Stellar Developers Documentation](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup).

- **Stellar Asset Contract (SAC)**: Deploy the SAC for the Stellar asset intended to be used in the contract using the Stellar CLI. Refer to the [Deploy the Stellar Asset Contract for a Stellar asset](https://developers.stellar.org/docs/build/guides/cli/deploy-stellar-asset-contract) guide for instructions.

### Testing

The contract includes 19 comprehensive tests covering all functionality and error scenarios:

1. **test_initialization** — Contract setup with admin and token addresses.
2. **test_reinitialization** — Prevents duplicate initialization (`Error #2`).
3. **test_create_proposal** — Valid proposal creation with time constraints.
4. **test_start_time_after_end** — Timing validation (`Error #9`).
5. **test_start_time_in_past** — Past start time rejection (`Error #10`).
6. **test_duration_too_long** — Maximum duration enforcement (`Error #11`).
7. **test_duration_too_short** — Minimum duration enforcement (`Error #12`).
8. **test_proposal_already_exists** — Duplicate proposal prevention (`Error #3`).
9. **test_vote** — Successful voting with all three available choices.
10. **test_vote_boundary_inclusive** — Validates inclusive voting at start and end times.
11. **test_proposal_not_found** — Non-existent proposal voting (`Error #4`).
12. **test_user_already_voted** — Duplicate vote prevention (`Error #5`).
13. **test_user_cannot_vote** — Token-gated access control (`Error #6`).
14. **test_voting_not_active** — Timing constraint enforcement (`Error #7`).
15. **test_invalid_choice** — Invalid vote option rejection (`Error #8`).
16. **test_transfer_admin** — Admin privilege transfer.
17. **test_get_governance_details** — Proposal list retrieval.
18. **test_get_proposal_details** — Individual proposal data.
19. **test_get_user_details** — User voting history and eligibility.

- Run the complete test suite:

  ```bash
  cargo test
  ```

- For verbose output:

  ```bash
  cargo test -- --nocapture
  ```

- Run a specific test:

  ```bash
  cargo test test_vote
  ```

### Usage

- **Build**: Compile the contract to WASM for deployment.

  ```bash
  stellar contract build
  ```

- `__constructor`: Deploy and initialize with admin and token addresses.

  ```bash
  stellar contract deploy \
  --wasm target/wasm32v1-none/release/token_gated_vote_contract.wasm \
  --source <DEPLOYER_PRIVATE_KEY> \
  --network testnet \
  -- \
  --admin <ADMIN_PUBLIC_KEY> \
  --token <STELLAR_ASSET_CONTRACT>
  ```

- `create_proposal`: Create a new proposal (admin only, 5-15 day duration).

  ```bash
  stellar contract invoke \
  --id <TOKEN_GATED_VOTE_CONTRACT_ID> \
  --source <ADMIN_PRIVATE_KEY> \
  --network testnet \
  -- \
  create_proposal \
  --id <"SYMBOL"> \
  --description <"STRING"> \
  --start_time <UNIX_TIMESTAMP> \
  --end_time <UNIX_TIMESTAMP>
  ```

- `vote`: Cast a vote (requires token balance > 0, equal weight per holder).

  ```bash
  stellar contract invoke \
  --id <TOKEN_GATED_VOTE_CONTRACT_ID> \
  --source <CALLER_PRIVATE_KEY> \
  --network testnet \
  -- \
  vote \
  --user <CALLER_PUBLIC_KEY> \
  --id <"SYMBOL"> \
  --choice <"SYMBOL">
  ```

- `transfer_admin`: Transfer admin privileges (current admin only).

  ```bash
  stellar contract invoke \
  --id <TOKEN_GATED_VOTE_CONTRACT_ID> \
  --source <ADMIN_PRIVATE_KEY> \
  --network testnet \
  -- \
  transfer_admin \
  --new_admin <NEW_ADMIN_PUBLIC_KEY>
  ```

- `get_governance_details`: Get all proposal summaries.

  ```bash
  stellar contract invoke \
  --id <TOKEN_GATED_VOTE_CONTRACT_ID> \
  --source <CALLER_PRIVATE_KEY> \
  --network testnet \
  -- \
  get_governance_details
  ```

- `get_proposal_details`: Get specific proposal data including vote counts.

  ```bash
  stellar contract invoke \
  --id <TOKEN_GATED_VOTE_CONTRACT_ID> \
  --source <CALLER_PRIVATE_KEY> \
  --network testnet \
  -- \
  get_proposal_details \
  --id <"SYMBOL">
  ```

- `get_user_details`: Get user voting history and eligibility.

  ```bash
  stellar contract invoke \
  --id <TOKEN_GATED_VOTE_CONTRACT_ID> \
  --source <CALLER_PRIVATE_KEY> \
  --network testnet \
  -- \
  get_user_details \
  --user <CALLER_PUBLIC_KEY>
  ```

## Contributing

If you're interested in helping improve the `pg-contracts` project or this particular contract, please see the [CONTRIBUTING](/CONTRIBUTING.md) file for guidelines on how to get started.

## License

This project is licensed under the [MIT License](/LICENSE).
