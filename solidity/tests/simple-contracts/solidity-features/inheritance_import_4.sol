pragma abicoder v1;

import {A, B} from "./inheritance_import_1.sol";

contract C is B {
    function test() public returns (uint) {
        return foo(new A());
    }
}
