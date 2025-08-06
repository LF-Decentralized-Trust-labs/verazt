pragma solidity ^0.8.1;

// Base contract
contract Parent {
    uint internal sum;

    function setValue() external {
        uint a = 10;
        uint b = 20;
        sum = a + b;
    }
}

// Inherited contract
contract Child is Parent {
    function getValue() external view returns (uint) {
        return sum;
    }
}

// Defining calling contract
contract Caller {
    // Creating child contract object
    Child cc = new Child();

    // Defining function to call
    // setValue and getValue functions
    function testInheritance() public returns (uint) {
        cc.setValue();
        return cc.getValue();
    }
}
