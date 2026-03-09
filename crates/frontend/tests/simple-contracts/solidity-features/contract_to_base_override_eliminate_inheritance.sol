// SPDX-License-Identifier: UNLICENSED

contract A {
    uint value;
    // function foo() public virtual returns (uint256) {
    //     return 1;
    // }

    // function setValue(uint _value) public {
    //     value = _value;
    // }
}

function __A__foo(A a) returns (uint256) {
    return 1;
}

function __A__setValue(A a, uint _value) {
    // ERROR: cannot assign to a contract's variables
    a.value = _value;
}



contract B is A {
    // function foo() public override returns (uint256) {
    //     return A.foo();
    // }

    // function bar() public returns (uint256) {
    //     return 2;
    // }
}

function __B__foo(B b) returns (uint256) {
    return __A__foo(b);
}

function __B__bar(B b) returns (uint256) {
    return 2;
}

contract C {
    function f() public {
        A a1 = new A();
        A a2 = new B();
        B b = new B();
        A a3 = b;
        B b2 = b;

        // uint256 x = a3.foo();
        // uint256 x = __A__foo(a3);

        // uint256 y = b2.bar();

        a1;
    }
}
