pragma solidity 0.8.15;

library SomeLibrary {
    function add(uint256 self, uint256 b) public returns (uint256) {
        return self + b;
    }
}

library Foo {
    function add(uint256 a, uint256 b) public returns (uint256) {
        return a + b;
    }
}

contract SomeContract {
    function add3(uint256 number) public returns (uint256) {
        return SomeLibrary.add(number, 3);
    }
    
    function add3_2(uint256 number) public returns (uint256) {
        return Foo.add(number, 3);
    }
}
