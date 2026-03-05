/// BUG: Missing capability check on emergency_withdraw.
/// Any signer can call emergency_withdraw and drain the vault.
module vault::vault_buggy {
    use std::signer;
    use aptos_framework::coin::{Self, Coin};

    struct Vault<phantom CoinType> has key {
        balance: Coin<CoinType>,
        admin: address,
    }

    const E_INSUFFICIENT_BALANCE: u64 = 2;

    public entry fun initialize<CoinType>(admin: &signer) {
        let admin_addr = signer::address_of(admin);
        move_to(admin, Vault<CoinType> {
            balance: coin::zero<CoinType>(),
            admin: admin_addr,
        });
    }

    public entry fun deposit<CoinType>(
        user: &signer,
        vault_addr: address,
        amount: u64,
    ) acquires Vault {
        let coin = coin::withdraw<CoinType>(user, amount);
        let vault = borrow_global_mut<Vault<CoinType>>(vault_addr);
        coin::merge(&mut vault.balance, coin);
    }

    // BUG: Missing admin check — any signer can call this and drain the vault.
    // The vault.admin field is never compared against the caller's address.
    public entry fun emergency_withdraw<CoinType>(
        caller: &signer,
        vault_addr: address,
    ) acquires Vault {
        let caller_addr = signer::address_of(caller);
        let vault = borrow_global_mut<Vault<CoinType>>(vault_addr);
        // BUG: no assertion that caller_addr == vault.admin
        let amount = coin::value(&vault.balance);
        let coin = coin::extract(&mut vault.balance, amount);
        coin::deposit(caller_addr, coin);
    }
}
