import {a as b, fre as fre2} from "same_constants_different_files_s1.sol";
import "same_constants_different_files_s1.sol" as M;

uint256 constant a = 13;

contract C {
    function f() public returns (uint, uint, uint, uint) {
        return (M.fre(), fre2(), M.a, b);
    }
}
