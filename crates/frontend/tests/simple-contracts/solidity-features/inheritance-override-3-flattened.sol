// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

contract A {
    uint x;

    uint y;

    function foo() public pure virtual returns (string memory) {
        return "A";
    }
}

// Original contract B, which inherits A
contract B is A {
    // Override A.foo()
    function foo() public pure virtual override returns (string memory) {
        return "B";
    }

    function zoo() public view returns (uint) {
        return x + y;
    }
}

// Flattened contract: B_flattened
contract B_flattened /* is A */ {
    // Clone x from A
    uint x;

    // Clone y from A
    uint y;

    // Clone method of A
    function A__foo() public pure virtual returns (string memory) {
        return "A";
    }

    // The overridden method of B
    function foo() public pure /* virtual override */ returns (string memory) {
        return "B";
    }

    function zoo() public view returns (uint) {
        return x + y;
    }
}

// Original contract C, which inherits A
contract C is A {
    // Override A.foo()
    function foo() public pure virtual override returns (string memory) {
        return "C";
    }
}

// Flattened contract: C_flattened
contract C_flattened /* is A */ {
    // Clone x from A
    uint x;

    // Clone y from A
    uint y;

    // Clone method of A
    function A__foo() public pure virtual returns (string memory) {
        return "A";
    }

    // The overridden method of C
    function foo() public pure /* virtual override */ returns (string memory) {
        return "C";
    }
}

// Original contract D, which inherits B, C
contract D is B, C {
    function foo() public pure override(B, C) returns (string memory) {
        return super.foo();     // here call C.foo()
    }

    function bar() public pure returns (string memory) {
        return A.foo();
    }

    function zar() public view returns (uint) {
        return zoo();
    }
}

// Flattened contract: D_flattened
contract D_flattened /* is B, C */ {
    // Clone x from A
    uint x;

    // Clone y from A
    uint y;

    // Clone method of A
    function A__foo() public pure virtual returns (string memory) {
        return "A";
    }

    // Clone method of B
    function B__foo() public pure virtual returns (string memory) {
        return "B";
    }

    // Copy method of C
    function C__foo() public pure /* virtual override */ returns (string memory) {
        return "C";
    }

    function foo() public pure /* override(B, C) */ returns (string memory) {
        return C__foo();     // here call C.foo()
    }

    function bar() public pure returns (string memory) {
        return A__foo();
    }

    function zoo() public view returns (uint) {
        return x + y;
    }

    function zar() public view returns (uint) {
        return zoo();
    }
}

contract E is B, C {
    // Overriding foo
    function foo() public pure override(C, B) returns (string memory) {
        return super.foo();  // call B.foo()
    }

    function bar() public pure returns (string memory) {
        return foo();
    }
}

contract E_flattened /* is B, C */ {
    // Clone method of A
    function A__foo() public pure virtual returns (string memory) {
        return "A";
    }

    // Clone method of B
    function B__foo() public pure virtual returns (string memory) {
        return "B";
    }

    // Copy method of C
    function C__foo() public pure /* virtual override */ returns (string memory) {
        return "C";
    }

    // Overriding foo
    function foo() public pure /* override(C, B) */ returns (string memory) {
        return B__foo();  // call B.foo()
    }

    function bar() public pure returns (string memory) {
        return foo();
    }
}

// Swapping A, B will throw an error
contract F is A, B {
    function foo() public pure override(A, B) returns (string memory) {
        return super.foo();
    }

    function bar_1() public pure returns (string memory) {
        return A.foo();
    }

    function bar_2() public pure returns (string memory) {
        return B.foo();
    }
}

// Swapping A, B will throw an error
contract F_flattened /* is A, B */ {
    // Clone method of A
    function A__foo() public pure virtual returns (string memory) {
        return "A";
    }

    // Clone method of B
    function B__foo() public pure virtual returns (string memory) {
        return "B";
    }

    function foo() public pure /* override(A, B) */ returns (string memory) {
        return B__foo();
    }

    function bar_1() public pure returns (string memory) {
        return A__foo();
    }

    function bar_2() public pure returns (string memory) {
        return B__foo();
    }
}


contract G {
    function foo() public pure virtual returns (string memory) {
        return "G";
    }
}

// Contracts inherit other contracts by using the keyword 'is'.
contract H is G {
    // Override G.foo()
    function foo() public pure virtual override returns (string memory) {
        return "H";
    }
}

contract I is G {
    // Override G.foo()
    function foo() public pure virtual override returns (string memory) {
        return "I";
    }
}
