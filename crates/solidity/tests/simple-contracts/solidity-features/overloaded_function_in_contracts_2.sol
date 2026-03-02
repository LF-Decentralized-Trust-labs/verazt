contract C {
    function g() public returns (uint, uint) {
        return (f(2), f("abc"));
    }

    function g(uint x) public returns (uint) {
        return x;
    }

    function z(uint x) public returns (uint) {
        A a;
        uint b = g(x);
        string memory f = a.f("ABC");
        return a.f(b);
    }
}

contract A {
    function f(uint a) public returns (uint) {
        return a;
    }

    function f(string memory a) public returns (string memory) {
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
