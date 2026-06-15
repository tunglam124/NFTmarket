#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, vec,
    Address, Env, String, Vec, Map,
};

// ───────────────────────────────────────────────
// DATA TYPES
// ───────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct NFT {
    pub id: u64,
    pub owner: Address,
    pub creator: Address,
    pub title: String,
    pub uri: String,       // IPFS metadata URI
    pub price: i128,       // in stroops (1 XLM = 10_000_000)
    pub for_sale: bool,
    pub royalty_bps: u32,  // e.g. 500 = 5%
}

#[contracttype]
#[derive(Clone)]
pub struct Auction {
    pub nft_id: u64,
    pub seller: Address,
    pub highest_bidder: Option<Address>,
    pub highest_bid: i128,
    pub min_bid: i128,
    pub end_time: u64,
    pub active: bool,
}

#[contracttype]
pub enum Key {
    Nft(u64),
    Auction(u64),
    OwnerNfts(Address),
    Escrow(Address),
    Counter,
    Admin,
    FeeBps,
}

// ───────────────────────────────────────────────
// CONTRACT
// ───────────────────────────────────────────────

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {

    // ── Init ──────────────────────────────────

    pub fn init(env: Env, admin: Address, fee_bps: u32) {
        admin.require_auth();
        env.storage().instance().set(&Key::Admin, &admin);
        env.storage().instance().set(&Key::FeeBps, &fee_bps);
        env.storage().instance().set(&Key::Counter, &0u64);
    }

    // ── Mint ──────────────────────────────────

    pub fn mint(
        env: Env,
        creator: Address,
        title: String,
        uri: String,
        price: i128,
        royalty_bps: u32,
    ) -> u64 {
        creator.require_auth();

        let id: u64 = env.storage().instance().get(&Key::Counter).unwrap_or(0) + 1;
        env.storage().instance().set(&Key::Counter, &id);

        let nft = NFT { id, owner: creator.clone(), creator: creator.clone(),
                        title, uri, price, for_sale: false, royalty_bps };

        env.storage().persistent().set(&Key::Nft(id), &nft);
        Self::push_owner_nft(&env, &creator, id);
        id
    }

    // ── List / Delist ─────────────────────────

    pub fn list(env: Env, owner: Address, nft_id: u64, price: i128) {
        owner.require_auth();
        let mut nft: NFT = env.storage().persistent().get(&Key::Nft(nft_id)).unwrap();
        assert!(nft.owner == owner, "not owner");
        nft.price = price;
        nft.for_sale = true;
        env.storage().persistent().set(&Key::Nft(nft_id), &nft);
    }

    pub fn delist(env: Env, owner: Address, nft_id: u64) {
        owner.require_auth();
        let mut nft: NFT = env.storage().persistent().get(&Key::Nft(nft_id)).unwrap();
        assert!(nft.owner == owner, "not owner");
        nft.for_sale = false;
        env.storage().persistent().set(&Key::Nft(nft_id), &nft);
    }

    // ── Buy ───────────────────────────────────

    pub fn buy(env: Env, buyer: Address, nft_id: u64) {
        buyer.require_auth();

        let mut nft: NFT = env.storage().persistent().get(&Key::Nft(nft_id)).unwrap();
        assert!(nft.for_sale,       "not for sale");
        assert!(nft.owner != buyer, "cannot buy own NFT");

        let balance = Self::escrow_balance(&env, &buyer);
        assert!(balance >= nft.price, "insufficient escrow balance");

        let fee_bps: u32 = env.storage().instance().get(&Key::FeeBps).unwrap_or(250);
        let platform  = (nft.price * fee_bps as i128) / 10_000;
        let royalty   = (nft.price * nft.royalty_bps as i128) / 10_000;
        let seller_cut = nft.price - platform - royalty;

        // Deduct buyer
        env.storage().persistent().set(&Key::Escrow(buyer.clone()), &(balance - nft.price));

        // Pay seller, creator, admin
        Self::credit(&env, &nft.owner,    seller_cut);
        Self::credit(&env, &nft.creator,  royalty);
        let admin: Address = env.storage().instance().get(&Key::Admin).unwrap();
        Self::credit(&env, &admin, platform);

        // Transfer ownership
        Self::pop_owner_nft(&env, &nft.owner, nft_id);
        nft.owner    = buyer.clone();
        nft.for_sale = false;
        env.storage().persistent().set(&Key::Nft(nft_id), &nft);
        Self::push_owner_nft(&env, &buyer, nft_id);
    }

    // ── Transfer (gift) ───────────────────────

    pub fn transfer(env: Env, from: Address, to: Address, nft_id: u64) {
        from.require_auth();
        let mut nft: NFT = env.storage().persistent().get(&Key::Nft(nft_id)).unwrap();
        assert!(nft.owner == from, "not owner");
        Self::pop_owner_nft(&env, &from, nft_id);
        nft.owner    = to.clone();
        nft.for_sale = false;
        env.storage().persistent().set(&Key::Nft(nft_id), &nft);
        Self::push_owner_nft(&env, &to, nft_id);
    }

    // ── Auction ───────────────────────────────

    pub fn start_auction(env: Env, seller: Address, nft_id: u64, min_bid: i128, end_time: u64) {
        seller.require_auth();
        let nft: NFT = env.storage().persistent().get(&Key::Nft(nft_id)).unwrap();
        assert!(nft.owner == seller, "not owner");

        let auction = Auction { nft_id, seller, highest_bidder: None,
                                highest_bid: 0, min_bid, end_time, active: true };
        env.storage().persistent().set(&Key::Auction(nft_id), &auction);
    }

    pub fn bid(env: Env, bidder: Address, nft_id: u64, amount: i128) {
        bidder.require_auth();
        let mut auction: Auction = env.storage().persistent().get(&Key::Auction(nft_id)).unwrap();
        assert!(auction.active, "auction not active");
        assert!(env.ledger().timestamp() < auction.end_time, "auction ended");
        assert!(amount > auction.highest_bid && amount >= auction.min_bid, "bid too low");

        let balance = Self::escrow_balance(&env, &bidder);
        assert!(balance >= amount, "insufficient escrow balance");

        // Refund previous bidder
        if let Some(prev) = &auction.highest_bidder {
            Self::credit(&env, prev, auction.highest_bid);
        }

        // Lock new bidder's funds
        env.storage().persistent().set(&Key::Escrow(bidder.clone()), &(balance - amount));

        auction.highest_bidder = Some(bidder);
        auction.highest_bid    = amount;
        env.storage().persistent().set(&Key::Auction(nft_id), &auction);
    }

    pub fn settle(env: Env, nft_id: u64) {
        let mut auction: Auction = env.storage().persistent().get(&Key::Auction(nft_id)).unwrap();
        assert!(auction.active, "already settled");
        assert!(env.ledger().timestamp() >= auction.end_time, "auction still running");

        auction.active = false;
        env.storage().persistent().set(&Key::Auction(nft_id), &auction);

        let winner = match auction.highest_bidder {
            None    => return, // no bids — NFT stays with seller
            Some(w) => w,
        };

        let mut nft: NFT = env.storage().persistent().get(&Key::Nft(nft_id)).unwrap();
        let fee_bps: u32  = env.storage().instance().get(&Key::FeeBps).unwrap_or(250);
        let platform      = (auction.highest_bid * fee_bps as i128) / 10_000;
        let royalty       = (auction.highest_bid * nft.royalty_bps as i128) / 10_000;
        let seller_cut    = auction.highest_bid - platform - royalty;

        Self::credit(&env, &auction.seller, seller_cut);
        Self::credit(&env, &nft.creator,    royalty);
        let admin: Address = env.storage().instance().get(&Key::Admin).unwrap();
        Self::credit(&env, &admin, platform);

        Self::pop_owner_nft(&env, &nft.owner, nft_id);
        nft.owner    = winner.clone();
        nft.for_sale = false;
        env.storage().persistent().set(&Key::Nft(nft_id), &nft);
        Self::push_owner_nft(&env, &winner, nft_id);
    }

    // ── Escrow ────────────────────────────────

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        let bal = Self::escrow_balance(&env, &user);
        env.storage().persistent().set(&Key::Escrow(user), &(bal + amount));
    }

    pub fn withdraw(env: Env, user: Address, amount: i128) {
        user.require_auth();
        let bal = Self::escrow_balance(&env, &user);
        assert!(bal >= amount, "insufficient balance");
        env.storage().persistent().set(&Key::Escrow(user), &(bal - amount));
    }

    // ── Read ──────────────────────────────────

    pub fn get_nft(env: Env, nft_id: u64) -> NFT {
        env.storage().persistent().get(&Key::Nft(nft_id)).unwrap()
    }

    pub fn get_auction(env: Env, nft_id: u64) -> Auction {
        env.storage().persistent().get(&Key::Auction(nft_id)).unwrap()
    }

    pub fn get_owner_nfts(env: Env, owner: Address) -> Vec<u64> {
        env.storage().persistent()
            .get(&Key::OwnerNfts(owner))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        Self::escrow_balance(&env, &user)
    }

    pub fn get_total(env: Env) -> u64 {
        env.storage().instance().get(&Key::Counter).unwrap_or(0)
    }

    // ── Helpers ───────────────────────────────

    fn escrow_balance(env: &Env, user: &Address) -> i128 {
        env.storage().persistent().get(&Key::Escrow(user.clone())).unwrap_or(0)
    }

    fn credit(env: &Env, to: &Address, amount: i128) {
        if amount <= 0 { return; }
        let bal = Self::escrow_balance(env, to);
        env.storage().persistent().set(&Key::Escrow(to.clone()), &(bal + amount));
    }

    fn push_owner_nft(env: &Env, owner: &Address, nft_id: u64) {
        let mut list: Vec<u64> = env.storage().persistent()
            .get(&Key::OwnerNfts(owner.clone()))
            .unwrap_or_else(|| Vec::new(env));
        list.push_back(nft_id);
        env.storage().persistent().set(&Key::OwnerNfts(owner.clone()), &list);
    }

    fn pop_owner_nft(env: &Env, owner: &Address, nft_id: u64) {
        let list: Vec<u64> = env.storage().persistent()
            .get(&Key::OwnerNfts(owner.clone()))
            .unwrap_or_else(|| Vec::new(env));
        let mut new_list: Vec<u64> = Vec::new(env);
        for id in list.iter() {
            if id != nft_id { new_list.push_back(id); }
        }
        env.storage().persistent().set(&Key::OwnerNfts(owner.clone()), &new_list);
    }
}

mod test;