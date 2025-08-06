pragma solidity ^0.8.0;

contract A {
  struct Foo { uint y; }

  function b1(function(Foo memory) external returns (uint)[] storage) internal pure {}
  function c1(function(Foo memory) external returns (uint)[] memory) public pure {}
  function d1(function(Foo memory) external returns (uint)[] calldata) external pure {}
}
