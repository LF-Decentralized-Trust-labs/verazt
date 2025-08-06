function f(uint a, uint b, uint c) pure returns (uint) {
    return a + b + c + 2;
}
function g(uint x) pure returns (uint) {
    return x + 8;
}

using {g, f} for uint;

contract C {
    function test(uint x, uint y) public pure returns (uint, uint) {
        uint t = x.f({a: 2, c: 3});
        return (x.f({c: 3, b: 2}), y.g());
    }
}
