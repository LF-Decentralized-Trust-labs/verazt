pragma solidity ^0.5.0;

contract SolidityTest {
    uint storedData;
    constructor() public{
        storedData = 10;
    }
    function getResult() public view returns(string memory){
        uint n = 1;
        uint sum = 0;

        while( n < 10){
            n++;
            if(n == 5){
                continue; // skip n in sum when it is 5.
            }
            sum = sum + n;
        }
        return integerToString(sum);
    }
    function integerToString(uint _i) internal pure
        returns (string memory) {

        if (_i == 0) {
            return "0";
        }
        uint j = _i;
        uint len;

        while (true) {
            len++;
            j /= 10;
            if(j==0){
                break;   //using break statement
            }
        }
        bytes memory bstr = new bytes(len);
        uint k = len - 1;

        while (_i != 0) {
            bstr[k--] = byte(uint8(48 + _i % 10));
            _i /= 10;
        }
        return string(bstr);
    }
}
