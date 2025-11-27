#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol, Vec,
};

const RLP_NS: Symbol = symbol_short!("RLP");

#[contracttype]
#[derive(Clone)]
pub struct RecipientShare {
    pub recipient: Address,
    pub bps: u32, // 10000 = 100%
}

#[contracttype]
#[derive(Clone)]
pub struct RoyaltyStream {
    pub stream_id: u64,
    pub owner: Address,
    pub total_bps: u32,
    pub active: bool,
}

#[contract]
pub struct RoyaltyLinkProtocol;

#[contractimpl]
impl RoyaltyLinkProtocol {
    // NOTE: owner is now an argument, no env.invoker()
    pub fn create_stream(env: Env, stream_id: u64, owner: Address, recipients: Vec<RecipientShare>) {
        let inst = env.storage().instance();

        let s_key = Self::stream_key(stream_id);
        if inst.has(&s_key) {
            panic!("stream already exists");
        }

        let mut total: u32 = 0;
        for r in recipients.iter() {
            total = total
                .checked_add(r.bps)
                .unwrap_or_else(|| panic!("bps overflow"));
        }
        if total != 10_000 {
            panic!("total bps must equal 10000");
        }

        let stream = RoyaltyStream {
            stream_id,
            owner: owner.clone(),
            total_bps: total,
            active: true,
        };

        let r_key = Self::recipients_key(stream_id);
        inst.set(&s_key, &stream);
        inst.set(&r_key, &recipients);
    }

    // toggle_stream now also takes caller Address instead of env.invoker()
    pub fn toggle_stream(env: Env, stream_id: u64, caller: Address, active: bool) {
        let inst = env.storage().instance();

        let s_key = Self::stream_key(stream_id);
        let mut s: RoyaltyStream =
            inst.get(&s_key).unwrap_or_else(|| panic!("stream not found"));

        if s.owner != caller {
            panic!("only owner");
        }

        s.active = active;
        inst.set(&s_key, &s);
    }

    pub fn calc_shares(env: Env, stream_id: u64, amount: i128) -> Vec<(Address, i128)> {
        if amount <= 0 {
            panic!("amount must be positive");
        }

        let inst = env.storage().instance();
        let s_key = Self::stream_key(stream_id);
        let r_key = Self::recipients_key(stream_id);

        let s: RoyaltyStream =
            inst.get(&s_key).unwrap_or_else(|| panic!("stream not found"));
        if !s.active {
            panic!("inactive stream");
        }

        let recipients: Vec<RecipientShare> =
            inst.get(&r_key).unwrap_or_else(|| panic!("recipients missing"));

        let mut out: Vec<(Address, i128)> = Vec::new(&env);
        for r in recipients.iter() {
            let share = (amount * (r.bps as i128)) / 10_000;
            out.push_back((r.recipient.clone(), share));
        }
        out
    }

    pub fn get_stream(env: Env, stream_id: u64) -> Option<RoyaltyStream> {
        let inst = env.storage().instance();
        let key = Self::stream_key(stream_id);
        inst.get(&key)
    }

    pub fn get_recipients(env: Env, stream_id: u64) -> Option<Vec<RecipientShare>> {
        let inst = env.storage().instance();
        let key = Self::recipients_key(stream_id);
        inst.get(&key)
    }

    fn stream_key(id: u64) -> (Symbol, u64) {
        (RLP_NS, id)
    }

    fn recipients_key(id: u64) -> (Symbol, Symbol, u64) {
        (RLP_NS, symbol_short!("RCPS"), id)
    }
}
