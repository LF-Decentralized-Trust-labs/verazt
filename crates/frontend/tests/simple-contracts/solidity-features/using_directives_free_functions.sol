function f(uint x) pure returns (uint) {
    return x + 2;
}
function g(uint x) pure returns (uint) {
    return x + 8;
}

using {g, f} for uint;

contract C {
    function test(uint x, uint y) public pure returns (uint, uint) {
        return (x.f(), y.g());
    }
}
