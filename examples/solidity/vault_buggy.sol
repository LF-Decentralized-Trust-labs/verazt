// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

/// @title VaultBuggy — intentional reentrancy vulnerability.
/// @notice The withdraw function makes the external call BEFORE updating
///         the balance, violating the Checks-Effects-Interactions pattern.
contract VaultBuggy {
    IERC20 public token;
    address public owner;
    mapping(address => uint256) public balances;

    event Deposit(address indexed user, uint256 amount);
    event Withdraw(address indexed user, uint256 amount);

    constructor(address _token) {
        token = IERC20(_token);
        owner = msg.sender;
    }

    function deposit(uint256 amount) external {
        require(amount > 0, "Amount must be > 0");
        require(token.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        balances[msg.sender] += amount;
        emit Deposit(msg.sender, amount);
    }

    // BUG: external call before state update — classic reentrancy vulnerability.
    // An attacker contract can re-enter withdraw() before balances[msg.sender]
    // is decremented, draining the vault.
    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        // VULNERABLE: external call before balance update
        require(token.transfer(msg.sender, amount), "Transfer failed");
        balances[msg.sender] -= amount;
        emit Withdraw(msg.sender, amount);
    }
}
