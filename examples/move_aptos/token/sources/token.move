/// A simple coin-like module using aptos_framework::coin.
module token::token {
    use std::signer;
    use std::string;
    use aptos_framework::coin::{Self, MintCapability, BurnCapability};

    /// Marker type for this coin.
    struct TOKEN {}

    /// Stored at the admin's address, holds mint/burn capabilities.
    struct Capabilities has key {
        mint_cap: MintCapability<TOKEN>,
        burn_cap: BurnCapability<TOKEN>,
    }

    /// Initialize the coin type with metadata and store capabilities.
    public entry fun initialize(account: &signer) {
        let (burn_cap, freeze_cap, mint_cap) = coin::initialize<TOKEN>(
            account,
            string::utf8(b"Token"),
            string::utf8(b"TKN"),
            8,
            true,
        );
        coin::destroy_freeze_cap(freeze_cap);
        move_to(account, Capabilities { mint_cap, burn_cap });
    }

    /// Mint coins and deposit them to the caller's account.
    public entry fun mint(account: &signer, amount: u64) acquires Capabilities {
        let addr = signer::address_of(account);
        let caps = borrow_global<Capabilities>(addr);
        let coins = coin::mint<TOKEN>(amount, &caps.mint_cap);
        coin::deposit(addr, coins);
    }

    /// Transfer coins from the caller to the recipient.
    public entry fun transfer(from: &signer, to: address, amount: u64) {
        coin::transfer<TOKEN>(from, to, amount);
    }
}
