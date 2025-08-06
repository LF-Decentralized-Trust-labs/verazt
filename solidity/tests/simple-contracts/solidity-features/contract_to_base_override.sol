// SPDX-License-Identifier: UNLICENSED

contract A {
    function foo() virtual public returns (uint) {
        return 1;
    }
}

contract B is A {
    function foo() override public returns (uint) {
        // return 2;
        return A.foo();
    }

    function bar() public returns (uint) {
        return 2;
    }
}

contract C {
  function f() public {
    A a1 = new A();
    A a2 = new B();
    B b = new B();
    A a3 = b;
    B b2 = b;

    uint x = a3.foo();
    uint y = b2.bar();

    a1;
  }
}
