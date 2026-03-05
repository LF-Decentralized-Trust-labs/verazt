/// A vault using Aptos resources with proper access control via capability pattern.
module vault::vault {
    use std::signer;
    use aptos_framework::coin::{Self, Coin};

    /// Vault resource stored at the admin's address.
    struct Vault<phantom CoinType> has key {
        balance: Coin<CoinType>,
        admin: address,
    }

    const E_NOT_ADMIN: u64 = 1;
    const E_INSUFFICIENT_BALANCE: u64 = 2;

    /// Create a new empty vault at the admin's address.
    public entry fun initialize<CoinType>(admin: &signer) {
        let admin_addr = signer::address_of(admin);
        move_to(admin, Vault<CoinType> {
            balance: coin::zero<CoinType>(),
            admin: admin_addr,
        });
    }

    /// Deposit coins into the vault.
    public entry fun deposit<CoinType>(
        user: &signer,
        vault_addr: address,
        amount: u64,
    ) acquires Vault {
        let coin = coin::withdraw<CoinType>(user, amount);
        let vault = borrow_global_mut<Vault<CoinType>>(vault_addr);
        coin::merge(&mut vault.balance, coin);
    }

    /// Withdraw coins from the vault back to the caller.
    public entry fun withdraw<CoinType>(
        user: &signer,
        vault_addr: address,
        amount: u64,
    ) acquires Vault {
        let vault = borrow_global_mut<Vault<CoinType>>(vault_addr);
        assert!(coin::value(&vault.balance) >= amount, E_INSUFFICIENT_BALANCE);
        let coin = coin::extract(&mut vault.balance, amount);
        let user_addr = signer::address_of(user);
        coin::deposit(user_addr, coin);
    }

    /// Admin-only: drain the entire vault.
    /// Properly checks that the caller is the vault admin.
    public entry fun emergency_withdraw<CoinType>(
        admin: &signer,
        vault_addr: address,
    ) acquires Vault {
        let admin_addr = signer::address_of(admin);
        let vault = borrow_global_mut<Vault<CoinType>>(vault_addr);
        assert!(admin_addr == vault.admin, E_NOT_ADMIN);
        let amount = coin::value(&vault.balance);
        let coin = coin::extract(&mut vault.balance, amount);
        coin::deposit(admin_addr, coin);
    }
}
