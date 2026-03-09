contract C {
    function g() public returns (uint, uint) {
        return (f(2), f("abc"));
    }

    function g(uint x) public returns (uint) {
        return x;
    }

    function z(uint x) public returns (uint) {
        uint a = g(x);
        return a;
    }
}

function f(uint) returns (uint) {
    return 2;
}

function f(string memory) returns (uint) {
    return 3;
}

function g(bool) returns (uint) {
    return 1;
}
