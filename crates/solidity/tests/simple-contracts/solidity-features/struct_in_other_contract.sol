pragma solidity ^0.8.0;

contract A {
  struct Foo { uint y; }

  function b1(function(Foo memory) external returns (uint)[] storage) internal pure {}
  function c1(function(Foo memory) external returns (uint)[] memory) public pure {}
  function d1(function(Foo memory) external returns (uint)[] calldata) external pure {}
}

contract B {
    struct Bar { uint y; }
}

contract C {
    // ensure that we consider array of function pointers as reference type
    function b2(function(B.Bar memory) external returns (uint)[] storage) internal pure {}
    function c2(function(B.Bar memory) external returns (uint)[] memory) public pure {}
    function d2(function(B.Bar memory) external returns (uint)[] calldata) external pure {}
}
