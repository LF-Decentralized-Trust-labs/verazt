# @version ^0.3.9

# @title Minimal ERC-20 Token in Vyper
# @notice A canonical example of a simple fungible token contract.

name: public(String[32])
symbol: public(String[8])
decimals: public(uint8)
totalSupply: public(uint256)

balanceOf: public(HashMap[address, uint256])
allowance: public(HashMap[address, HashMap[address, uint256]])

event Transfer:
    sender: indexed(address)
    receiver: indexed(address)
    amount: uint256

event Approval:
    owner: indexed(address)
    spender: indexed(address)
    amount: uint256

@deploy
def __init__(
    _name: String[32],
    _symbol: String[8],
    _decimals: uint8,
    _initial_supply: uint256,
):
    self.name = _name
    self.symbol = _symbol
    self.decimals = _decimals
    self.totalSupply = _initial_supply
    self.balanceOf[msg.sender] = _initial_supply
    log Transfer(empty(address), msg.sender, _initial_supply)

@external
def transfer(to: address, amount: uint256) -> bool:
    assert self.balanceOf[msg.sender] >= amount, "Insufficient balance"
    self.balanceOf[msg.sender] -= amount
    self.balanceOf[to] += amount
    log Transfer(msg.sender, to, amount)
    return True

@external
def approve(spender: address, amount: uint256) -> bool:
    self.allowance[msg.sender][spender] = amount
    log Approval(msg.sender, spender, amount)
    return True

@external
def transferFrom(sender: address, to: address, amount: uint256) -> bool:
    assert self.balanceOf[sender] >= amount, "Insufficient balance"
    assert self.allowance[sender][msg.sender] >= amount, "Insufficient allowance"
    self.allowance[sender][msg.sender] -= amount
    self.balanceOf[sender] -= amount
    self.balanceOf[to] += amount
    log Transfer(sender, to, amount)
    return True
