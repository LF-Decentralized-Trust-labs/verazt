
/// @use-src 0:"timelock_solidity_v0_8.sol"
object "TimeLockVault_96" {
    code {
        /// @src 0:464:1455  "contract TimeLockVault {..."
        mstore(64, memoryguard(128))
        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

        constructor_TimeLockVault_96()

        let _1 := allocate_unbounded()
        codecopy(_1, dataoffset("TimeLockVault_96_deployed"), datasize("TimeLockVault_96_deployed"))

        return(_1, datasize("TimeLockVault_96_deployed"))

        function allocate_unbounded() -> memPtr {
            memPtr := mload(64)
        }

        function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
            revert(0, 0)
        }

        /// @src 0:464:1455  "contract TimeLockVault {..."
        function constructor_TimeLockVault_96() {

            /// @src 0:464:1455  "contract TimeLockVault {..."

        }
        /// @src 0:464:1455  "contract TimeLockVault {..."

    }
    /// @use-src 0:"timelock_solidity_v0_8.sol"
    object "TimeLockVault_96_deployed" {
        code {
            /// @src 0:464:1455  "contract TimeLockVault {..."
            mstore(64, memoryguard(128))

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0x27e235e3
                {
                    // balances(address)

                    external_fun_balances_5()
                }

                case 0x3ccfd60b
                {
                    // withdraw()

                    external_fun_withdraw_95()
                }

                case 0x79af55e4
                {
                    // increaseLockTime(uint256)

                    external_fun_increaseLockTime_49()
                }

                case 0xa4beda63
                {
                    // lockTime(address)

                    external_fun_lockTime_9()
                }

                case 0xd0e30db0
                {
                    // deposit()

                    external_fun_deposit_31()
                }

                default {}
            }

            revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74()

            function shift_right_224_unsigned(value) -> newValue {
                newValue :=

                shr(224, value)

            }

            function allocate_unbounded() -> memPtr {
                memPtr := mload(64)
            }

            function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
                revert(0, 0)
            }

            function revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() {
                revert(0, 0)
            }

            function revert_error_c1322bf8034eace5e0b5c7295db60986aa89aae5e0ea0873e4689e076861a5db() {
                revert(0, 0)
            }

            function cleanup_t_uint160(value) -> cleaned {
                cleaned := and(value, 0xffffffffffffffffffffffffffffffffffffffff)
            }

            function cleanup_t_address(value) -> cleaned {
                cleaned := cleanup_t_uint160(value)
            }

            function validator_revert_t_address(value) {
                if iszero(eq(value, cleanup_t_address(value))) { revert(0, 0) }
            }

            function abi_decode_t_address(offset, end) -> value {
                value := calldataload(offset)
                validator_revert_t_address(value)
            }

            function abi_decode_tuple_t_address(headStart, dataEnd) -> value0 {
                if slt(sub(dataEnd, headStart), 32) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

            }

            function identity(value) -> ret {
                ret := value
            }

            function convert_t_uint160_to_t_uint160(value) -> converted {
                converted := cleanup_t_uint160(identity(cleanup_t_uint160(value)))
            }

            function convert_t_uint160_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_uint160(value)
            }

            function convert_t_address_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_address(value)
            }

            function mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(slot , key) -> dataSlot {
                mstore(0, convert_t_address_to_t_address(key))
                mstore(0x20, slot)
                dataSlot := keccak256(0, 0x40)
            }

            function shift_right_unsigned_dynamic(bits, value) -> newValue {
                newValue :=

                shr(bits, value)

            }

            function cleanup_from_storage_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function extract_from_storage_value_dynamict_uint256(slot_value, offset) -> value {
                value := cleanup_from_storage_t_uint256(shift_right_unsigned_dynamic(mul(offset, 8), slot_value))
            }

            function read_from_storage_split_dynamic_t_uint256(slot, offset) -> value {
                value := extract_from_storage_value_dynamict_uint256(sload(slot), offset)

            }

            /// @ast-id 5
            /// @src 0:548:588  "mapping(address => uint) public balances"
            function getter_fun_balances_5(key_0) -> ret {

                let slot := 0
                let offset := 0

                slot := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(slot, key_0)

                ret := read_from_storage_split_dynamic_t_uint256(slot, offset)

            }
            /// @src 0:464:1455  "contract TimeLockVault {..."

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function abi_encode_t_uint256_to_t_uint256_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint256(value))
            }

            function abi_encode_tuple_t_uint256__to_t_uint256__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_uint256_to_t_uint256_fromStack(value0,  add(headStart, 0))

            }

            function external_fun_balances_5() {

                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                let param_0 :=  abi_decode_tuple_t_address(4, calldatasize())
                let ret_0 :=  getter_fun_balances_5(param_0)
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                return(memPos, sub(memEnd, memPos))

            }

            function abi_decode_tuple_(headStart, dataEnd)   {
                if slt(sub(dataEnd, headStart), 0) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

            }

            function abi_encode_tuple__to__fromStack(headStart ) -> tail {
                tail := add(headStart, 0)

            }

            function external_fun_withdraw_95() {

                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                abi_decode_tuple_(4, calldatasize())
                fun_withdraw_95()
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                return(memPos, sub(memEnd, memPos))

            }

            function validator_revert_t_uint256(value) {
                if iszero(eq(value, cleanup_t_uint256(value))) { revert(0, 0) }
            }

            function abi_decode_t_uint256(offset, end) -> value {
                value := calldataload(offset)
                validator_revert_t_uint256(value)
            }

            function abi_decode_tuple_t_uint256(headStart, dataEnd) -> value0 {
                if slt(sub(dataEnd, headStart), 32) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_uint256(add(headStart, offset), dataEnd)
                }

            }

            function external_fun_increaseLockTime_49() {

                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                let param_0 :=  abi_decode_tuple_t_uint256(4, calldatasize())
                fun_increaseLockTime_49(param_0)
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                return(memPos, sub(memEnd, memPos))

            }

            /// @ast-id 9
            /// @src 0:595:635  "mapping(address => uint) public lockTime"
            function getter_fun_lockTime_9(key_0) -> ret {

                let slot := 1
                let offset := 0

                slot := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(slot, key_0)

                ret := read_from_storage_split_dynamic_t_uint256(slot, offset)

            }
            /// @src 0:464:1455  "contract TimeLockVault {..."

            function external_fun_lockTime_9() {

                if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                let param_0 :=  abi_decode_tuple_t_address(4, calldatasize())
                let ret_0 :=  getter_fun_lockTime_9(param_0)
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                return(memPos, sub(memEnd, memPos))

            }

            function external_fun_deposit_31() {

                abi_decode_tuple_(4, calldatasize())
                fun_deposit_31()
                let memPos := allocate_unbounded()
                let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                return(memPos, sub(memEnd, memPos))

            }

            function revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74() {
                revert(0, 0)
            }

            function shift_right_0_unsigned(value) -> newValue {
                newValue :=

                shr(0, value)

            }

            function extract_from_storage_value_offset_0t_uint256(slot_value) -> value {
                value := cleanup_from_storage_t_uint256(shift_right_0_unsigned(slot_value))
            }

            function read_from_storage_split_offset_0_t_uint256(slot) -> value {
                value := extract_from_storage_value_offset_0t_uint256(sload(slot))

            }

            function panic_error_0x11() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x11)
                revert(0, 0x24)
            }

            function checked_add_t_uint256(x, y) -> sum {
                x := cleanup_t_uint256(x)
                y := cleanup_t_uint256(y)

                // overflow, if x > (maxValue - y)
                if gt(x, sub(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, y)) { panic_error_0x11() }

                sum := add(x, y)
            }

            function shift_left_0(value) -> newValue {
                newValue :=

                shl(0, value)

            }

            function update_byte_slice_32_shift_0(value, toInsert) -> result {
                let mask := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                toInsert := shift_left_0(toInsert)
                value := and(value, not(mask))
                result := or(value, and(toInsert, mask))
            }

            function convert_t_uint256_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_uint256(value)))
            }

            function prepare_store_t_uint256(value) -> ret {
                ret := value
            }

            function update_storage_value_offset_0t_uint256_to_t_uint256(slot, value_0) {
                let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
                sstore(slot, update_byte_slice_32_shift_0(sload(slot), prepare_store_t_uint256(convertedValue_0)))
            }

            function cleanup_t_rational_31536000_by_1(value) -> cleaned {
                cleaned := value
            }

            function convert_t_rational_31536000_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_rational_31536000_by_1(value)))
            }

            /// @ast-id 31
            /// @src 0:674:820  "function deposit() public payable {..."
            function fun_deposit_31() {

                /// @src 0:743:752  "msg.value"
                let expr_17 := callvalue()
                /// @src 0:719:727  "balances"
                let _1 := 0x00
                let expr_12 := _1
                /// @src 0:728:738  "msg.sender"
                let expr_14 := caller()
                /// @src 0:719:739  "balances[msg.sender]"
                let _2 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_12,expr_14)
                /// @src 0:719:752  "balances[msg.sender] += msg.value"
                let _3 := read_from_storage_split_offset_0_t_uint256(_2)
                let expr_18 := checked_add_t_uint256(_3, expr_17)

                update_storage_value_offset_0t_uint256_to_t_uint256(_2, expr_18)
                /// @src 0:786:801  "block.timestamp"
                let expr_25 := timestamp()
                /// @src 0:804:812  "365 days"
                let expr_26 := 0x01e13380
                /// @src 0:786:812  "block.timestamp + 365 days"
                let expr_27 := checked_add_t_uint256(expr_25, convert_t_rational_31536000_by_1_to_t_uint256(expr_26))

                /// @src 0:763:771  "lockTime"
                let _4 := 0x01
                let expr_20 := _4
                /// @src 0:772:782  "msg.sender"
                let expr_22 := caller()
                /// @src 0:763:783  "lockTime[msg.sender]"
                let _5 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_20,expr_22)
                /// @src 0:763:812  "lockTime[msg.sender] = block.timestamp + 365 days"
                update_storage_value_offset_0t_uint256_to_t_uint256(_5, expr_27)
                let expr_28 := expr_27

            }
            /// @src 0:464:1455  "contract TimeLockVault {..."

            /// @ast-id 49
            /// @src 0:963:1102  "function increaseLockTime(uint secondsToIncrease) public {..."
            function fun_increaseLockTime_49(var_secondsToIncrease_33) {

                /// @src 0:1054:1062  "lockTime"
                let _6 := 0x01
                let expr_40 := _6
                /// @src 0:1063:1073  "msg.sender"
                let expr_42 := caller()
                /// @src 0:1054:1074  "lockTime[msg.sender]"
                let _7 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_40,expr_42)
                let _8 := read_from_storage_split_offset_0_t_uint256(_7)
                let expr_43 := _8
                /// @src 0:1077:1094  "secondsToIncrease"
                let _9 := var_secondsToIncrease_33
                let expr_44 := _9
                /// @src 0:1054:1094  "lockTime[msg.sender] + secondsToIncrease"
                let expr_45 := checked_add_t_uint256(expr_43, expr_44)

                /// @src 0:1031:1039  "lockTime"
                let _10 := 0x01
                let expr_36 := _10
                /// @src 0:1040:1050  "msg.sender"
                let expr_38 := caller()
                /// @src 0:1031:1051  "lockTime[msg.sender]"
                let _11 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_36,expr_38)
                /// @src 0:1031:1094  "lockTime[msg.sender] = lockTime[msg.sender] + secondsToIncrease"
                update_storage_value_offset_0t_uint256_to_t_uint256(_11, expr_45)
                let expr_46 := expr_45

            }
            /// @src 0:464:1455  "contract TimeLockVault {..."

            function cleanup_t_rational_0_by_1(value) -> cleaned {
                cleaned := value
            }

            function convert_t_rational_0_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_rational_0_by_1(value)))
            }

            function require_helper(condition) {
                if iszero(condition) { revert(0, 0) }
            }

            function convert_t_uint160_to_t_address_payable(value) -> converted {
                converted := convert_t_uint160_to_t_uint160(value)
            }

            function convert_t_address_to_t_address_payable(value) -> converted {
                converted := convert_t_uint160_to_t_address_payable(value)
            }

            function convert_t_address_payable_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_address(value)
            }

            function revert_forward_1() {
                let pos := allocate_unbounded()
                returndatacopy(pos, 0, returndatasize())
                revert(pos, returndatasize())
            }

            /// @ast-id 95
            /// @src 0:1174:1452  "function withdraw() public {..."
            function fun_withdraw_95() {

                /// @src 0:1220:1228  "balances"
                let _12 := 0x00
                let expr_53 := _12
                /// @src 0:1229:1239  "msg.sender"
                let expr_55 := caller()
                /// @src 0:1220:1240  "balances[msg.sender]"
                let _13 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_53,expr_55)
                let _14 := read_from_storage_split_offset_0_t_uint256(_13)
                let expr_56 := _14
                /// @src 0:1243:1244  "0"
                let expr_57 := 0x00
                /// @src 0:1220:1244  "balances[msg.sender] > 0"
                let expr_58 := gt(cleanup_t_uint256(expr_56), convert_t_rational_0_by_1_to_t_uint256(expr_57))
                /// @src 0:1212:1245  "require(balances[msg.sender] > 0)"
                require_helper(expr_58)
                /// @src 0:1264:1279  "block.timestamp"
                let expr_63 := timestamp()
                /// @src 0:1282:1290  "lockTime"
                let _15 := 0x01
                let expr_64 := _15
                /// @src 0:1291:1301  "msg.sender"
                let expr_66 := caller()
                /// @src 0:1282:1302  "lockTime[msg.sender]"
                let _16 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_64,expr_66)
                let _17 := read_from_storage_split_offset_0_t_uint256(_16)
                let expr_67 := _17
                /// @src 0:1264:1302  "block.timestamp > lockTime[msg.sender]"
                let expr_68 := gt(cleanup_t_uint256(expr_63), cleanup_t_uint256(expr_67))
                /// @src 0:1256:1303  "require(block.timestamp > lockTime[msg.sender])"
                require_helper(expr_68)
                /// @src 0:1335:1343  "balances"
                let _18 := 0x00
                let expr_73 := _18
                /// @src 0:1344:1354  "msg.sender"
                let expr_75 := caller()
                /// @src 0:1335:1355  "balances[msg.sender]"
                let _19 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_73,expr_75)
                let _20 := read_from_storage_split_offset_0_t_uint256(_19)
                let expr_76 := _20
                /// @src 0:1314:1355  "uint transferValue = balances[msg.sender]"
                let var_transferValue_72 := expr_76
                /// @src 0:1389:1390  "0"
                let expr_82 := 0x00
                /// @src 0:1366:1390  "balances[msg.sender] = 0"
                let _21 := convert_t_rational_0_by_1_to_t_uint256(expr_82)
                /// @src 0:1366:1374  "balances"
                let _22 := 0x00
                let expr_78 := _22
                /// @src 0:1375:1385  "msg.sender"
                let expr_80 := caller()
                /// @src 0:1366:1386  "balances[msg.sender]"
                let _23 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_78,expr_80)
                /// @src 0:1366:1390  "balances[msg.sender] = 0"
                update_storage_value_offset_0t_uint256_to_t_uint256(_23, _21)
                let expr_83 := _21
                /// @src 0:1409:1419  "msg.sender"
                let expr_88 := caller()
                /// @src 0:1401:1420  "payable(msg.sender)"
                let expr_89 := convert_t_address_to_t_address_payable(expr_88)
                /// @src 0:1401:1429  "payable(msg.sender).transfer"
                let expr_90_address := convert_t_address_payable_to_t_address(expr_89)
                /// @src 0:1430:1443  "transferValue"
                let _24 := var_transferValue_72
                let expr_91 := _24
                /// @src 0:1401:1444  "payable(msg.sender).transfer(transferValue)"

                let _25 := 0
                if iszero(expr_91) { _25 := 2300 }
                let _26 := call(_25, expr_90_address, expr_91, 0, 0, 0, 0)

                if iszero(_26) { revert_forward_1() }

            }
            /// @src 0:464:1455  "contract TimeLockVault {..."

        }

        data ".metadata" hex"a26469706673582212208678a1f30f56325fccfb39ca95973c3f77c3fd1e7a4ef91ffd9396b2fccc07dc64736f6c634300080f0033"
    }

  {
    // Load the length (first 32 bytes)
    let len := mload(data)

    // Skip over the length field.
    //
    // Keep temporary variable so it can be incremented in place.
    //
    // NOTE: incrementing data would result in an unusable
    //       data variable after this assembly block
    let dataElementLocation := add(data, 0x20)

    // Iterate until the bound is not met.
    for { let end := add(dataElementLocation, mul(len, 0x20)) }
        lt(dataElementLocation, end)
        { dataElementLocation := add(dataElementLocation, 0x20) }
    {
      sum := add(sum, mload(dataElementLocation))
    }
  }
}
