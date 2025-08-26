#![no_std]

use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Symbol, Vec,
};

// --- Vote Choice Constants ---
const VOTE_FOR: Symbol = symbol_short!("FOR");
const VOTE_AGAINST: Symbol = symbol_short!("AGAINST");
const VOTE_ABSTAIN: Symbol = symbol_short!("ABSTAIN");

// --- Proposal Duration Constraints (in seconds) ---
const MAX_PROPOSAL_DURATION: u64 = 1292000; // ~15 days
const MIN_PROPOSAL_DURATION: u64 = 432000; // ~5 days

// --- Storage Time-To-Live (TTL) Constants (in ledger seconds) ---
const PROPOSALS_TTL_EXTENSION: u32 = 2_100_000; // ~24 days
const PROPOSAL_TTL_BUFFER: u32 = 604_800; // ~7 days
const VOTE_TTL_EXTENSION: u32 = 1_600_000; // ~18.5 days

// Defines the structure for persistent and instance storage
#[contracttype]
pub enum TokenGatedVoteContractDataKey {
    Admin,            // Contract administrator address
    Token,            // Governance token address
    Proposal(Symbol), // Individual proposal data, keyed by its ID
    Proposals,        // List of all proposal IDs
    Votes(Address),   // User voting records
}

// Stores the detailed information for a single proposal
#[contracttype]
#[derive(Clone)]
pub struct TokenGatedVoteProposalData {
    pub description: String, // Proposal description
    pub start_time: u64,     // UNIX timestamp when voting begins
    pub end_time: u64,       // UNIX timestamp when voting ends
    pub total_for: i128,     // Total voting power cast FOR
    pub total_against: i128, // Total voting power cast AGAINST
    pub total_abstain: i128, // Total voting power cast ABSTAIN
}

// Represents a summary of a governance proposal
#[contracttype]
#[derive(Clone)]
pub struct TokenGatedVoteProposalSummary {
    pub id: Symbol,                           // Unique identifier for the proposal
    pub description: String,                  // Human-readable proposal description
    pub status: TokenGatedVoteProposalStatus, // Lifecycle status of the proposal
}

// Represents lifecycle status of a proposal relative to the current ledger timestamp
#[contracttype]
#[derive(Clone, Copy)]
pub enum TokenGatedVoteProposalStatus {
    Pending, // Current time is before start_time
    Active,  // Current time is within [start_time, end_time]
    Ended,   // Current time is after end_time
}

// Enumerates the possible error states for the contract
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenGatedVoteContractErrors {
    ContractNotInitialized = 1,     // The contract has not been initialized
    ContractAlreadyInitialized = 2, // The contract has already been initialized
    ProposalAlreadyExists = 3,      // A proposal with this ID already exists
    ProposalNotFound = 4,           // The specified proposal does not exist
    UserAlreadyVoted = 5,           // User has already voted on this proposal
    UserCannotVote = 6,             // User does not hold the required token
    VotingNotActive = 7,            // The proposal is not currently active for voting
    InvalidChoice = 8,              // The provided vote choice is invalid
    StartTimeAfterEnd = 9,          // Proposal start time occurs after end time
    StartTimeInPast = 10,           // Proposal start time is before current timestamp
    DurationTooLong = 11,           // Proposal duration exceeds maximum allowed period
    DurationTooShort = 12,          // Proposal duration is below minimum required period
}

#[contract]
pub struct TokenGatedVoteContract;

#[contractimpl]
impl TokenGatedVoteContract {
    // --- Helper Functions ---

    // Derives TTL extension for a proposal based on current ledger time
    fn calculate_proposal_ttl(env: &Env, proposal_end_time: u64) -> u32 {
        let ledger_time = env.ledger().timestamp();
        let proposal_duration = if proposal_end_time > ledger_time {
            proposal_end_time - ledger_time
        } else {
            0
        };

        let min_ttl = proposal_duration as u32 + PROPOSAL_TTL_BUFFER;
        min_ttl.max(PROPOSALS_TTL_EXTENSION)
    }

    // Computes proposal status relative to a ledger timestamp
    fn compute_proposal_status(
        ledger_time: u64,
        proposal: &TokenGatedVoteProposalData,
    ) -> TokenGatedVoteProposalStatus {
        if ledger_time < proposal.start_time {
            TokenGatedVoteProposalStatus::Pending
        } else if ledger_time <= proposal.end_time {
            TokenGatedVoteProposalStatus::Active
        } else {
            TokenGatedVoteProposalStatus::Ended
        }
    }

    // Validates proposal start/end times against ledger time and duration bounds
    fn validate_proposal_times(
        ledger_time: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<(), TokenGatedVoteContractErrors> {
        if start_time >= end_time {
            return Err(TokenGatedVoteContractErrors::StartTimeAfterEnd);
        }
        if start_time < ledger_time {
            return Err(TokenGatedVoteContractErrors::StartTimeInPast);
        }
        let duration = end_time - start_time;
        if duration > MAX_PROPOSAL_DURATION {
            return Err(TokenGatedVoteContractErrors::DurationTooLong);
        }
        if duration < MIN_PROPOSAL_DURATION {
            return Err(TokenGatedVoteContractErrors::DurationTooShort);
        }
        Ok(())
    }

    // --- Write Functions ---

    // Initializes contract with admin and governance token
    pub fn __constructor(
        env: Env,
        admin: Address,
        token: Address,
    ) -> Result<(), TokenGatedVoteContractErrors> {
        if env
            .storage()
            .instance()
            .has(&TokenGatedVoteContractDataKey::Admin)
        {
            return Err(TokenGatedVoteContractErrors::ContractAlreadyInitialized);
        }
        env.storage()
            .instance()
            .set(&TokenGatedVoteContractDataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&TokenGatedVoteContractDataKey::Token, &token);
        Ok(())
    }

    // Creates a proposal after validating timing and uniqueness
    pub fn create_proposal(
        env: Env,
        id: Symbol,
        description: String,
        start_time: u64,
        end_time: u64,
    ) -> Result<(), TokenGatedVoteContractErrors> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&TokenGatedVoteContractDataKey::Admin)
            .ok_or(TokenGatedVoteContractErrors::ContractNotInitialized)?;
        admin.require_auth();
        let ledger_time = env.ledger().timestamp();
        Self::validate_proposal_times(ledger_time, start_time, end_time)?;

        let proposal_key = TokenGatedVoteContractDataKey::Proposal(id.clone());
        if env.storage().persistent().has(&proposal_key) {
            return Err(TokenGatedVoteContractErrors::ProposalAlreadyExists);
        }

        let proposal = TokenGatedVoteProposalData {
            description,
            start_time,
            end_time,
            total_for: 0,
            total_against: 0,
            total_abstain: 0,
        };
        env.storage().persistent().set(&proposal_key, &proposal);

        let proposal_ttl = Self::calculate_proposal_ttl(&env, end_time);
        env.storage()
            .persistent()
            .extend_ttl(&proposal_key, proposal_ttl, proposal_ttl);

        let mut proposals: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&TokenGatedVoteContractDataKey::Proposals)
            .unwrap_or(Vec::new(&env));
        proposals.push_back(id.clone());
        env.storage()
            .persistent()
            .set(&TokenGatedVoteContractDataKey::Proposals, &proposals);

        env.storage().persistent().extend_ttl(
            &TokenGatedVoteContractDataKey::Proposals,
            PROPOSALS_TTL_EXTENSION,
            PROPOSALS_TTL_EXTENSION,
        );

        env.events().publish(("PROPOSAL", "CREATED"), id);
        Ok(())
    }

    // Records a user's vote on an active proposal after eligibility checks
    pub fn vote(
        env: Env,
        user: Address,
        id: Symbol,
        choice: Symbol,
    ) -> Result<(), TokenGatedVoteContractErrors> {
        user.require_auth();

        let proposal_key = TokenGatedVoteContractDataKey::Proposal(id.clone());
        let mut proposal: TokenGatedVoteProposalData = env
            .storage()
            .persistent()
            .get(&proposal_key)
            .ok_or(TokenGatedVoteContractErrors::ProposalNotFound)?;

        let ledger_time = env.ledger().timestamp();
        if ledger_time < proposal.start_time || ledger_time > proposal.end_time {
            return Err(TokenGatedVoteContractErrors::VotingNotActive);
        }

        let votes_key = TokenGatedVoteContractDataKey::Votes(user.clone());
        let mut votes: Map<Symbol, bool> = env
            .storage()
            .persistent()
            .get(&votes_key)
            .unwrap_or(Map::new(&env));

        if votes.contains_key(id.clone()) {
            return Err(TokenGatedVoteContractErrors::UserAlreadyVoted);
        }

        let token_address: Address = env
            .storage()
            .instance()
            .get(&TokenGatedVoteContractDataKey::Token)
            .ok_or(TokenGatedVoteContractErrors::ContractNotInitialized)?;
        let token_client = TokenClient::new(&env, &token_address);
        let token_balance = token_client.balance(&user);
        if token_balance <= 0 {
            return Err(TokenGatedVoteContractErrors::UserCannotVote);
        }

        if choice == VOTE_FOR {
            proposal.total_for = proposal.total_for.saturating_add(1);
        } else if choice == VOTE_AGAINST {
            proposal.total_against = proposal.total_against.saturating_add(1);
        } else if choice == VOTE_ABSTAIN {
            proposal.total_abstain = proposal.total_abstain.saturating_add(1);
        } else {
            return Err(TokenGatedVoteContractErrors::InvalidChoice);
        }

        votes.set(id.clone(), true);

        env.storage().persistent().set(&proposal_key, &proposal);
        env.storage().persistent().set(&votes_key, &votes);

        let proposal_ttl = Self::calculate_proposal_ttl(&env, proposal.end_time);
        env.storage()
            .persistent()
            .extend_ttl(&proposal_key, proposal_ttl, proposal_ttl);

        env.storage()
            .persistent()
            .extend_ttl(&votes_key, VOTE_TTL_EXTENSION, VOTE_TTL_EXTENSION);

        env.events().publish(("VOTE", id, user), (choice, 1));
        Ok(())
    }

    // Transfers admin role to a new address
    pub fn transfer_admin(
        env: Env,
        new_admin: Address,
    ) -> Result<(), TokenGatedVoteContractErrors> {
        let current_admin: Address = env
            .storage()
            .instance()
            .get(&TokenGatedVoteContractDataKey::Admin)
            .ok_or(TokenGatedVoteContractErrors::ContractNotInitialized)?;

        current_admin.require_auth();

        env.storage()
            .instance()
            .set(&TokenGatedVoteContractDataKey::Admin, &new_admin);

        env.events()
            .publish(("ADMIN", "TRANSFERRED"), (current_admin, new_admin));
        Ok(())
    }

    // --- Read-Only Functions ---

    // Returns summaries (id, description, status) for all proposals
    pub fn get_governance_details(env: Env) -> Vec<TokenGatedVoteProposalSummary> {
        let proposals: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&TokenGatedVoteContractDataKey::Proposals)
            .unwrap_or(Vec::new(&env));
        let mut summary = Vec::new(&env);

        let ledger_time = env.ledger().timestamp();

        for id in proposals.iter() {
            if let Some(proposal) = env
                .storage()
                .persistent()
                .get::<TokenGatedVoteContractDataKey, TokenGatedVoteProposalData>(
                    &TokenGatedVoteContractDataKey::Proposal(id.clone()),
                )
            {
                let status = Self::compute_proposal_status(ledger_time, &proposal);
                summary.push_back(TokenGatedVoteProposalSummary {
                    id: id.clone(),
                    description: proposal.description.clone(),
                    status,
                });
            }
        }
        summary
    }

    // Returns full stored data for a single proposal
    pub fn get_proposal_details(
        env: Env,
        id: Symbol,
    ) -> Result<TokenGatedVoteProposalData, TokenGatedVoteContractErrors> {
        let proposal: TokenGatedVoteProposalData = env
            .storage()
            .persistent()
            .get(&TokenGatedVoteContractDataKey::Proposal(id))
            .ok_or(TokenGatedVoteContractErrors::ProposalNotFound)?;
        Ok(proposal)
    }

    // Returns user's vote participation and eligibility per proposal
    pub fn get_user_details(
        env: Env,
        user: Address,
    ) -> Result<Vec<(Symbol, bool, i128)>, TokenGatedVoteContractErrors> {
        let proposals: Vec<Symbol> = env
            .storage()
            .persistent()
            .get(&TokenGatedVoteContractDataKey::Proposals)
            .unwrap_or(Vec::new(&env));

        let votes_key = TokenGatedVoteContractDataKey::Votes(user.clone());
        let votes: Map<Symbol, bool> = env
            .storage()
            .persistent()
            .get(&votes_key)
            .unwrap_or(Map::new(&env));

        let token_address: Address = env
            .storage()
            .instance()
            .get(&TokenGatedVoteContractDataKey::Token)
            .ok_or(TokenGatedVoteContractErrors::ContractNotInitialized)?;
        let token_client = TokenClient::new(&env, &token_address);
        let token_balance = token_client.balance(&user);

        let voting_power = if token_balance > 0 { 1 } else { 0 };

        let mut results = Vec::new(&env);
        for id in proposals.iter() {
            if let Some(_) = votes.get(id.clone()) {
                results.push_back((id.clone(), true, voting_power));
            } else {
                results.push_back((id.clone(), false, voting_power));
            }
        }
        Ok(results)
    }
}

// --- Test Module ---
mod test;
