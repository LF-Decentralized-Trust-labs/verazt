// SPDX-License-Identifier: MIT
pragma solidity ^0.8.1;

// import Foo.sol from current directory
import "./imported_contract.sol" as ABC;

import {Point as AliasName, add, Foo} from "./imported_contract.sol";
import * as NewLibrary from "./imported_contract.sol";
