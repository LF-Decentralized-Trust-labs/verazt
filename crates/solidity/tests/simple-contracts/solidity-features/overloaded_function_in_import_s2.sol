import {fre} from "overloaded_function_in_import_s1.sol";

// import "s1.sol" as M;

uint256 constant a = 13;

contract C {
  function f() public returns (uint256, uint256, uint256, uint256) {
    uint256 u = fre(10);
    return (a, fre(), u, a + u);
  }
}
