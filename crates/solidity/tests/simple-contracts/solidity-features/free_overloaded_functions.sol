pragma solidity ^0.8.0;

function add(uint a, uint b) pure returns (uint) {
  return a + b;
}

function add(uint a, uint b, uint c) pure returns (uint) {
  return a + b + c;
}

contract C {

  function subtract(uint a, uint b) internal pure returns (uint) {
    return a - b;
  }

  function f(uint x) public pure returns (uint) {
    return add(x, 2);
  }

  function h(uint x) public pure returns (uint) {
      return add(x, 2, 3);
  }

  function g(uint x) public pure returns (uint) {
    return subtract(x, 2);
  }
}

contract F {

    function subtract(uint a, uint b) internal pure returns (uint) {
        return a - b;
    }

    function add(uint a, uint b) internal pure returns (uint) {
        return a + b;
    }

    function f(uint x) public pure returns (uint) {
        return add(x, 2);
    }

    function f(uint x, uint y) public pure returns (uint) {
        return add(x, subtract(y, 2));
    }

    function g(uint x) public pure returns (uint) {
        return subtract(x, 2);
    }

    function h(uint x) public pure returns (uint) {
        return f(x, 2);
    }
}
