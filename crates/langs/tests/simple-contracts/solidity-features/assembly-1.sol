// SPDX-License-Identifier: MIT

pragma solidity 0.8.15;

/// @title Base64
/// @notice Provides a function for encoding some bytes in base64
/// @author Brecht Devos <[email protected]>
contract C {
    uint256 a = 1;

    function encode(uint x) internal pure returns (uint256) {

        uint256 a = 4 * (x / 3);

        if (x > 0) {
            uint256 a = x + 10;
            assembly {
                a := add(a, 1)
            }
        }

        a = a + 2;

        assembly {
            a := sub(a, 1)
        }

        return a;
    }
}
