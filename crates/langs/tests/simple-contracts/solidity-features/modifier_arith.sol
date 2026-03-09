// SPDX-License-Identifier: MIT
pragma solidity ^0.8.1;

contract C {
  modifier add(uint16 a, uint16 b) {
    unchecked { a + b; } // overflow not reported
    _;
  }

  function f(uint16 a, uint16 b, uint16 c) public pure add(a + b, c) returns (uint16) {
    return b + c; // can overflow
  }
}
