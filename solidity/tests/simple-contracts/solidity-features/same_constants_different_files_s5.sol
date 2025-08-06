uint256 constant a1 = a;

import {a as b, fre} from "same_constants_different_files_s1.sol";

uint256 constant a = 13;

import "same_constants_different_files_s1.sol" as M;

contract C {
    function f() public returns (uint, uint, uint, uint) {
        return (a, fre(), M.a, b);
    }
}
