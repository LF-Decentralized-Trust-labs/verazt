pragma solidity ^0.8.0;

struct Foo { uint y; }

contract A {
  function b1(function(Foo memory) external returns (uint)[] storage) internal pure {}
  function c1(function(Foo memory) external returns (uint)[] memory) public pure {}
  function d1(function(Foo memory) external returns (uint)[] calldata) external pure {}
}
