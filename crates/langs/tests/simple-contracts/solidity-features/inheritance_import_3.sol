pragma abicoder v1;

import * as SU from "./inheritance_import_1.sol";

contract C is SU.B {
    function test() public returns (uint) {
        return foo(new SU.A());
    }
}
