import { a as b, fre as fre2 } from "import_as_s1.sol";

uint256 constant a = 13;

function fre() pure returns (uint256) {
  return a;
}

contract C {
  function f() public returns (uint256, uint256, uint256, uint256) {
    return (a, fre(), fre2(),  b);
  }
}
