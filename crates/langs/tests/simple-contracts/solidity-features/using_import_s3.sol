import "using_import_s2.sol" as S2;

using {S2.S1.g, S2.S1.L.f} for uint;
using S2.S1.L2 for uint;

contract C {
    using S2.S1.L3 for *;

    struct St {
        uint value;
    }

    function test(uint x, uint y) public pure returns (uint, uint) {
        uint a = x.bar();
        return (x.f(), y.g());
    }

    function test2(uint x, uint8 y) public pure returns (uint, uint) {
        uint a = x.zar();
        return (x.f(), y.zar());
    }

    function test3(St memory x) public pure {
        uint a = x.value.zar();
    }
}
