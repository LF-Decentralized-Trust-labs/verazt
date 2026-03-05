# @version ^0.3.9

# @title Vault with reentrancy guard
# @notice Uses @nonreentrant("lock") on deposit/withdraw correctly.

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

event EmergencyWithdraw:
    admin: indexed(address)
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

@external
@nonreentrant("lock")
def withdraw(amount: uint256):
    assert self.balances[msg.sender] >= amount, "Insufficient balance"
    self.balances[msg.sender] -= amount
    assert self.token.transfer(msg.sender, amount), "Transfer failed"
    log Withdraw(msg.sender, amount)

@external
def emergencyWithdraw():
    assert msg.sender == self.owner, "Not owner"
    balance: uint256 = self.token.balanceOf(self)
    assert self.token.transfer(self.owner, balance), "Transfer failed"
    log EmergencyWithdraw(self.owner, balance)
