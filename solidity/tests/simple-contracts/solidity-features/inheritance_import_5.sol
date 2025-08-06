pragma abicoder v1;

import {B, A} from "./inheritance_import_1.sol";

contract C2 is B {
    function test() public returns (uint256) {
        return 1;
    }
}
