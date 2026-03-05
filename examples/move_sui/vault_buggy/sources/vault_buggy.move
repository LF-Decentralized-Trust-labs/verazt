/// BUG: admin_withdraw is missing the admin check entirely,
/// allowing any caller to drain the shared vault.
module vault::vault_buggy {
    use sui::object::{Self, UID};
    use sui::balance::{Self, Balance};
    use sui::coin::{Self, Coin};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use sui::sui::SUI;

    struct Vault has key {
        id: UID,
        balance: Balance<SUI>,
        admin: address,
    }

    fun init(ctx: &mut TxContext) {
        let vault = Vault {
            id: object::new(ctx),
            balance: balance::zero<SUI>(),
            admin: tx_context::sender(ctx),
        };
        transfer::share_object(vault);
    }

    public entry fun deposit(
        vault: &mut Vault,
        coin: Coin<SUI>,
        _ctx: &TxContext,
    ) {
        let coin_balance = coin::into_balance(coin);
        balance::join(&mut vault.balance, coin_balance);
    }

    // BUG: Missing admin check — any caller can drain the vault.
    // There is no assertion that tx_context::sender(ctx) == vault.admin.
    public entry fun admin_withdraw(
        vault: &mut Vault,
        ctx: &mut TxContext,
    ) {
        // BUG: no check that tx_context::sender(ctx) == vault.admin
        let amount = balance::value(&vault.balance);
        let withdrawn = balance::split(&mut vault.balance, amount);
        let coin = coin::from_balance(withdrawn, ctx);
        transfer::public_transfer(coin, tx_context::sender(ctx));
    }
}
