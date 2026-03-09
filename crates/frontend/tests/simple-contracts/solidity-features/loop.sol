// SPDX-License-Identifier: MIT
pragma solidity ^0.8.1;

contract Loop {

    function while_loop() public pure {
        // while loop
        uint k;
        while (k < 10) {
            k++;
        }
    }

    function while_loop2() public pure {
        // while loop
        uint k;
        while (k < 10)
            k++;
    }

    function do_while_loop() public pure {
        uint8 j = 0;

        do {
            j++;
        } while(j < 5) ;
    }

    function do_while_loop2() public pure {
        uint8 j = 0;

        do
            j++;

        while(j < 5) ;
    }
}

