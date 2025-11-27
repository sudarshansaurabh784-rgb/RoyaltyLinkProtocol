#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

const BOOTH_NS: Symbol = symbol_short!("BOOTH");

#[contracttype]
#[derive(Clone)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct Trade {
    pub trade_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub asset_desc: String,
    pub price: i128,
    pub status: TradeStatus,
}

#[contract]
pub struct SecureTradingBooth;

#[contractimpl]
impl SecureTradingBooth {
    // Seller creates a trade offer for a specific buyer
    pub fn create_trade(
        env: Env,
        trade_id: u64,
        seller: Address,
        buyer: Address,
        asset_desc: String,
        price: i128,
    ) {
        if price <= 0 {
            panic!("price must be positive");
        }

        let inst = env.storage().instance();
        let key = Self::trade_key(trade_id);

        if inst.has(&key) {
            panic!("trade_id exists");
        }

        let trade = Trade {
            trade_id,
            seller,
            buyer,
            asset_desc,
            price,
            status: TradeStatus::Created,
        };

        inst.set(&key, &trade);
    }

    // Buyer marks trade as funded (after paying off-chain or via token contract)
    pub fn mark_funded(env: Env, trade_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::trade_key(trade_id);

        let mut trade: Trade =
            inst.get(&key).unwrap_or_else(|| panic!("trade not found"));

        if caller != trade.buyer {
            panic!("only buyer can mark funded");
        }

        if let TradeStatus::Created = trade.status {
        } else {
            panic!("must be in Created state");
        }

        trade.status = TradeStatus::Funded;
        inst.set(&key, &trade);
    }

    // Seller confirms delivery and completes trade
    pub fn confirm_delivery(env: Env, trade_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::trade_key(trade_id);

        let mut trade: Trade =
            inst.get(&key).unwrap_or_else(|| panic!("trade not found"));

        if caller != trade.seller {
            panic!("only seller can confirm delivery");
        }

        if let TradeStatus::Funded = trade.status {
        } else {
            panic!("must be in Funded state");
        }

        trade.status = TradeStatus::Completed;
        inst.set(&key, &trade);
    }

    // Seller cancels trade before funding
    pub fn cancel_trade(env: Env, trade_id: u64, caller: Address) {
        let inst = env.storage().instance();
        let key = Self::trade_key(trade_id);

        let mut trade: Trade =
            inst.get(&key).unwrap_or_else(|| panic!("trade not found"));

        if caller != trade.seller {
            panic!("only seller can cancel");
        }

        if let TradeStatus::Created = trade.status {
        } else {
            panic!("can cancel only in Created state");
        }

        trade.status = TradeStatus::Cancelled;
        inst.set(&key, &trade);
    }

    // View helper
    pub fn get_trade(env: Env, trade_id: u64) -> Option<Trade> {
        let inst = env.storage().instance();
        let key = Self::trade_key(trade_id);
        inst.get(&key)
    }

    fn trade_key(id: u64) -> (Symbol, u64) {
        (BOOTH_NS, id)
    }
}
