// SPDX-License-Identifier: MIT
pragma solidity ^0.8.1;


contract Base1
{
    // uint256 x = 1;

    function foo() virtual public {}

}

contract Base2
{
    uint256 public x;
    uint256 y;
    function foo() virtual public {}
    function second() virtual public {}
}

contract Inherited is Base1, Base2
{
    uint256 public override x = 3;

    // Derives from multiple bases defining foo(), so we must explicitly
    // override it
    function foo() public override(Base1, Base2) {}

    function second() public override {}
}
