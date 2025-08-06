import {a as b, fre, fre as foo} from "import_as_s1.sol";

import "import_as_s1.sol" as S1;

uint256 constant a = 13;

contract C {
    function f() public returns (uint256, uint256, uint256, uint256) {
        uint256 n = foo();
        return (a, fre(), S1.fre(), b);
    }
}
