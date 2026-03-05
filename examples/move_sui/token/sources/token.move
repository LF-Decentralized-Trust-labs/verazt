/// A simple wrapped coin on Sui using the coin module.
module token::token {
    use sui::coin::{Self, TreasuryCap};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};

    /// One-time witness for the coin type.
    struct TOKEN has drop {}

    /// Called once on module publish. Creates the coin type and transfers
    /// the TreasuryCap to the publisher.
    fun init(witness: TOKEN, ctx: &mut TxContext) {
        let (treasury_cap, metadata) = coin::create_currency<TOKEN>(
            witness,
            8,                          // decimals
            b"TKN",                     // symbol
            b"Token",                   // name
            b"A simple token on Sui",   // description
            option::none(),             // icon URL
            ctx,
        );
        transfer::public_freeze_object(metadata);
        transfer::public_transfer(treasury_cap, tx_context::sender(ctx));
    }

    /// Mint new coins and transfer them to the recipient.
    public entry fun mint(
        treasury_cap: &mut TreasuryCap<TOKEN>,
        amount: u64,
        recipient: address,
        ctx: &mut TxContext,
    ) {
        let coin = coin::mint(treasury_cap, amount, ctx);
        transfer::public_transfer(coin, recipient);
    }

    /// Burn coins, reducing total supply.
    public entry fun burn(
        treasury_cap: &mut TreasuryCap<TOKEN>,
        coin: coin::Coin<TOKEN>,
    ) {
        coin::burn(treasury_cap, coin);
    }
}
