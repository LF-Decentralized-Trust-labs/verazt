// SPDX-License-Identifier: MIT
pragma solidity ^0.8.1;


contract Base1
{
    function foo() virtual public {}

}

contract Base2
{
    function foo() virtual public {}
    function second() virtual public {}
}

contract Inherited is Base1, Base2
{
    // Derives from multiple bases defining foo(), so we must explicitly
    // override it
    function foo() public override(Base1, Base2) {}

    function second() public override {
        Base1.foo();
    }

    function bar(Base1 base) public {
        base.foo();
    }
}
