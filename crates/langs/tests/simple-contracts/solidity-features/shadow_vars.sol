pragma solidity ^0.8.0;

contract Shadow {
    uint256 storedData; // State variable

    uint256 storedValue = 2; // State variable

    function getResult() public view returns (uint256) {
        uint256 a = 1; // local variable
        uint256 b = 2;
        uint256 result = a + b;

        {
            uint256 a = 2;
            uint256 b = 3;
            result = result + a + b;

        }

        if (a > b) {
            result++;
            --result;
        } else {
            result += 2;
        }

        result = result + 2;

        return storedData; //access the state variable
    }
}
