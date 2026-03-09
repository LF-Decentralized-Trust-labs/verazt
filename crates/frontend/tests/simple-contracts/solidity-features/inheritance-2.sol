struct Data {
    uint256 a;
    uint256[2] b;
    uint256 c;
}

contract A {
    function get() public view returns (Data memory) {
        return Data(5, [uint256(66), 77], 8);
    }
}

contract B {
    function foo(A _a) public returns (uint256) {
        return _a.get().b[1];
    }
}

contract C is B {
    function test() public returns (uint256) {
        return foo(new A());
    }
}
