// SPDX-License-Identifier: MIT
pragma solidity ^0.8.1;

contract Flatten {
    function test1() public pure {
        uint256 k;
        k = k + 1 + 2 - 3;
    }

    function test2() public pure {
        uint256 k;
        uint256 tmp__1 = k + 1;
        uint256 tmp__2 = tmp__1 + 2;
        k = tmp__2 - 3;
    }
}
