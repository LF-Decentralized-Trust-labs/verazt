# @version ^0.3.9

# @title Vault with MISSING reentrancy guard on withdraw
# @notice BUG: @nonreentrant is intentionally omitted from withdraw,
#         leaving it vulnerable to reentrancy attacks.

interface IERC20:
    def transfer(to: address, amount: uint256) -> bool: nonpayable
    def transferFrom(sender: address, to: address, amount: uint256) -> bool: nonpayable
    def balanceOf(account: address) -> uint256: view

token: public(IERC20)
owner: public(address)
balances: public(HashMap[address, uint256])

event Deposit:
    user: indexed(address)
    amount: uint256

event Withdraw:
    user: indexed(address)
    amount: uint256

@deploy
def __init__(_token: address):
    self.token = IERC20(_token)
    self.owner = msg.sender

@external
@nonreentrant("lock")
def deposit(amount: uint256):
    assert amount > 0, "Amount must be > 0"
    assert self.token.transferFrom(msg.sender, self, amount), "Transfer failed"
    self.balances[msg.sender] += amount
    log Deposit(msg.sender, amount)

# BUG: @nonreentrant decorator is intentionally omitted from withdraw.
# A malicious token contract can re-enter withdraw() during the transfer,
# draining the vault.
@external
def withdraw(amount: uint256):
    assert self.balances[msg.sender] >= amount, "Insufficient balance"
    self.balances[msg.sender] -= amount
    assert self.token.transfer(msg.sender, amount), "Transfer failed"
    log Withdraw(msg.sender, amount)
