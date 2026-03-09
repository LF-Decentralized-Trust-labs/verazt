pragma abicoder v1;

import "./inheritance_import_2.sol";

contract D is C {
    function test2() public returns (uint) {
        return foo(new A());
    }
}
