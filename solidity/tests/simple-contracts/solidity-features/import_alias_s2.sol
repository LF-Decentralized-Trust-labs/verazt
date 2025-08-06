import "import_alias_s1.sol" as M;

import {A as B} from "import_alias_s1.sol";

import {Color as Color2} from "import_alias_s1.sol";

enum ColorRGB {
    Red,
    Green,
    Blue
}

import {Editor as Editor2} from "import_alias_s1.sol";

struct Editor {
    string name;
    uint256 version;
    string executable;
}

uint256 constant a = 13;

contract A {
    function bar() public pure returns (uint256) {
        return 2;
    }

    function foo() public pure returns (uint256) {
        return bar();
    }
}

contract C {
    function f() public returns (uint256, uint256) {
        B b = new B();
        return (a, a);
    }
}
