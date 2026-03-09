pragma solidity 0.8.15;

contract C {
    function g() public returns (uint256, uint256) {
        return (f("abc"), f(2));
    }

    function g(uint256 x) public payable returns (uint256) {
        return x;
    }

    function z(uint256 x) public returns (uint256) {
        uint256 a = g(x);
        return a;
    }
}

function f(uint256) returns (uint256) {
    return 2;
}

function f(string memory) returns (uint256) {
    return 3;
}

function g(bool) returns (uint256) {
    return 1;
}
