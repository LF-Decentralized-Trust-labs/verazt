import "using_import_s1.sol" as S1;

using {S1.g, S1.L.f} for uint;

contract C {
    function test(uint x, uint y) public pure returns (uint, uint) {
        return (x.f(), y.g());
    }
}
