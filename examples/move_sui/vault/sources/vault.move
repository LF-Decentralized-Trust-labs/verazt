/// A vault using Sui shared objects with proper admin access control.
module vault::vault {
    use sui::object::{Self, UID};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::sui::SUI;

    /// Shared vault object holding SUI balance and admin address.
    struct Vault has key {
        id: UID,
        balance: Balance<SUI>,
        admin: address,
    }

    const E_NOT_ADMIN: u64 = 1;
    const E_INSUFFICIENT_BALANCE: u64 = 2;

    /// Create a shared vault. The deployer becomes the admin.
    fun init(ctx: &mut TxContext) {
        let vault = Vault {
            id: object::new(ctx),
            balance: balance::zero<SUI>(),
            admin: tx_context::sender(ctx),
        };
        transfer::share_object(vault);
    }

    /// Deposit SUI into the vault.
    public entry fun deposit(
        vault: &mut Vault,
        coin: Coin<SUI>,
        _ctx: &TxContext,
    ) {
        let coin_balance = coin::into_balance(coin);
        balance::join(&mut vault.balance, coin_balance);
    }

    /// Withdraw SUI from the vault.
    public entry fun withdraw(
        vault: &mut Vault,
        amount: u64,
        ctx: &mut TxContext,
    ) {
        assert!(balance::value(&vault.balance) >= amount, E_INSUFFICIENT_BALANCE);
        let withdrawn = balance::split(&mut vault.balance, amount);
        let coin = coin::from_balance(withdrawn, ctx);
        transfer::public_transfer(coin, tx_context::sender(ctx));
    }

    /// Admin-only: drain the entire vault.
    /// Properly checks that the caller is the vault admin.
    public entry fun admin_withdraw(
        vault: &mut Vault,
        ctx: &mut TxContext,
    ) {
        assert!(tx_context::sender(ctx) == vault.admin, E_NOT_ADMIN);
        let amount = balance::value(&vault.balance);
        let withdrawn = balance::split(&mut vault.balance, amount);
        let coin = coin::from_balance(withdrawn, ctx);
        transfer::public_transfer(coin, vault.admin);
    }
}
