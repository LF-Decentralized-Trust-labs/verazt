// This code is valid in Solidity

uint256 constant b = a;
uint256 constant a = 1;

contract Sample {
    function foo() public returns (uint256) {
        return b;
    }
}
