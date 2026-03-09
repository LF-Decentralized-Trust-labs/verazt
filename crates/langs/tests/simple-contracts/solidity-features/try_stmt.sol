// SPDX-License-Identifier: MIT
/* source: https://solidity-by-example.org/try-catch/ */
pragma solidity ^0.8.1;

// External contract used for try / catch examples
contract Foo {
    address public owner;

    constructor(address _owner) {
        require(_owner != address(0), "invalid address");
        assert(_owner != 0x0000000000000000000000000000000000000001);
        owner = _owner;
    }

    function myFunc(uint x) public pure returns (string memory) {
        require(x != 0, "require failed");
        return "my func was called";
    }
}

contract Bar {
    event Log(string message);
    event LogBytes(bytes data);

    Foo public foo;

    constructor() {
        // This Foo contract is used for example of try catch with external call
        foo = new Foo(msg.sender);
    }

    // Example of try / catch with external call
    // tryCatchExternalCall(0) => Log("external call failed")
    // tryCatchExternalCall(1) => Log("my func was called")
    function tryCatchExternalCall(uint _i) public {
        try foo.myFunc(_i) returns (string memory result) {
            emit Log(result);
        } catch {
            emit Log("external call failed");
        }
    }

}

contract WillThrow {
    function aFunction() public pure {
        require(false, "Error message");
    }
}

contract ErrorHandling {
    event ErrorLogging(string reason);
    function catchError() public {
        WillThrow will = new WillThrow();
        try will.aFunction() {
            //here we could do something if it works
        }  catch Error(string memory reason) {
            emit ErrorLogging(reason);
        }
    }
}

contract errorContract{
    function errorFunction() public payable returns(bool){
        revert("I fail always");
    }
}

contract tryCatch{
    bool    public success    = false;
    uint256 public errorCount = 0;
    string  public reason;
    errorContract eC;

    function testTryCatch(errorContract _errorContractAddress) public{
        eC = _errorContractAddress;
        try eC.errorFunction(){
            success = true;
        } catch Error (string memory _reason){
            success = false;
            errorCount++;
            reason = _reason;
        }
    }
}
