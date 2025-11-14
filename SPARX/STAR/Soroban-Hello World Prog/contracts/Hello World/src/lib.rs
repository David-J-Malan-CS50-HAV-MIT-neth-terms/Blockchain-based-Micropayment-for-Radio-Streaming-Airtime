#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to track listener's streaming session
#[contracttype]
#[derive(Clone)]
pub struct StreamingSession {
    pub listener: Address,
    pub total_paid: i128,        // Total tokens paid by listener
    pub minutes_streamed: u64,   // Total minutes of streaming
    pub is_active: bool,         // Current streaming status
    pub last_payment_time: u64,  // Timestamp of last payment
}

// Structure to track platform statistics
#[contracttype]
#[derive(Clone)]
pub struct PlatformStats {
    pub total_listeners: u64,
    pub total_revenue: i128,
    pub total_minutes: u64,
}

// Symbol for platform statistics
const PLATFORM_STATS: Symbol = symbol_short!("P_STATS");

// Enum for mapping listener address to their session
#[contracttype]
pub enum SessionBook {
    Session(Address)
}

#[contract]
pub struct StreamingPaymentContract;

#[contractimpl]
impl StreamingPaymentContract {

    // Function to start streaming session with micropayment
    pub fn start_streaming(env: Env, listener: Address, payment_amount: i128) {
        listener.require_auth();
        
        let mut session = Self::get_session(env.clone(), listener.clone());
        
        // Check if payment amount is sufficient (minimum 1 token)
        if payment_amount < 1 {
            log!(&env, "Payment amount too low!");
            panic!("Minimum payment is 1 token");
        }
        
        let time = env.ledger().timestamp();
        
        // Update session data
        session.listener = listener.clone();
        session.total_paid += payment_amount;
        session.is_active = true;
        session.last_payment_time = time;
        
        // Update platform statistics
        let mut stats = Self::get_platform_stats(env.clone());
        stats.total_revenue += payment_amount;
        
        // If new listener, increment count
        if session.minutes_streamed == 0 {
            stats.total_listeners += 1;
        }
        
        // Store updated data
        env.storage().instance().set(&SessionBook::Session(listener.clone()), &session);
        env.storage().instance().set(&PLATFORM_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Streaming started for listener. Payment: {} tokens", payment_amount);
    }

    // Function to add streaming minutes and stop session
    pub fn stop_streaming(env: Env, listener: Address, minutes: u64) {
        listener.require_auth();
        
        let mut session = Self::get_session(env.clone(), listener.clone());
        
        if !session.is_active {
            log!(&env, "No active streaming session found!");
            panic!("No active session");
        }
        
        // Update streaming minutes
        session.minutes_streamed += minutes;
        session.is_active = false;
        
        // Update platform statistics
        let mut stats = Self::get_platform_stats(env.clone());
        stats.total_minutes += minutes;
        
        // Store updated data
        env.storage().instance().set(&SessionBook::Session(listener.clone()), &session);
        env.storage().instance().set(&PLATFORM_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Streaming stopped. Total minutes: {}", session.minutes_streamed);
    }

    // Function to view listener's session details
    pub fn get_session(env: Env, listener: Address) -> StreamingSession {
        let key = SessionBook::Session(listener.clone());
        
        env.storage().instance().get(&key).unwrap_or(StreamingSession {
            listener: listener.clone(),
            total_paid: 0,
            minutes_streamed: 0,
            is_active: false,
            last_payment_time: 0,
        })
    }

    // Function to view platform statistics
    pub fn get_platform_stats(env: Env) -> PlatformStats {
        env.storage().instance().get(&PLATFORM_STATS).unwrap_or(PlatformStats {
            total_listeners: 0,
            total_revenue: 0,
            total_minutes: 0,
        })
    }
}