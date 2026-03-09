import "import_as_s2.sol" as S2;

uint256 constant a = 13;

contract C {
    function f() public returns (uint256, uint256, uint256, uint256) {
        uint256 n = S2.foo();
        return (a, S2.fre(), S2.S1.bar(), S2.b);
    }
}
