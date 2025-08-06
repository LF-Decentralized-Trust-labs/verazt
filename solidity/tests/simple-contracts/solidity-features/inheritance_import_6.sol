pragma abicoder v1;

import {B as B1, A as A1} from "./inheritance_import_1.sol";

contract C2 is B1 {
    function test() public returns (uint256) {
        return 1;
    }
}
