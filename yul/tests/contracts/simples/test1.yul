/// @use-src 0:"dao_solidity_v0_8.sol"
object "SimpleDAO_62" {
    code {
        /// @src 0:162:618  "contract SimpleDAO {..."
        mstore(64, 128)
        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

        constructor_SimpleDAO_62()

        let _1 := allocate_unbounded()
        codecopy(_1, dataoffset("SimpleDAO_62_deployed"), datasize("SimpleDAO_62_deployed"))

        return(_1, datasize("SimpleDAO_62_deployed"))

        function allocate_unbounded() -> memPtr {
            memPtr := mload(64)
        }

        function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
            revert(0, 0)
        }

        /// @src 0:162:618  "contract SimpleDAO {..."
        function constructor_SimpleDAO_62() {

            /// @src 0:162:618  "contract SimpleDAO {..."

        }
        /// @src 0:162:618  "contract SimpleDAO {..."

    }
    /// @use-src 0:"dao_solidity_v0_8.sol"
    object "SimpleDAO_62_deployed" {
        code {
            /// @src 0:162:618  "contract SimpleDAO {..."
            mstore(64, 128)

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0x00362a95
                {
                    // donate(address)

                    let param_0 :=  abi_decode_tuple_t_address(4, calldatasize())
                    fun_donate_18(param_0)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x2e1a7d4d
                {
                    // withdraw(uint256)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0 :=  abi_decode_tuple_t_uint256(4, calldatasize())
                    fun_withdraw_49(param_0)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x59f1286d
                {
                    // queryCredit(address)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0 :=  abi_decode_tuple_t_address(4, calldatasize())
                    let ret_0 :=  fun_queryCredit_61(param_0)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0xd5d44d80
                {
                    // credit(address)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0 :=  abi_decode_tuple_t_address(4, calldatasize())
                    let ret_0 :=  getter_fun_credit_5(param_0)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                default {}
            }
            if iszero(calldatasize()) {  }
            revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74()

            function shift_right_224_unsigned(value) -> newValue {
                newValue := shr(224, value)

            }

            function allocate_unbounded() -> memPtr {
                memPtr := mload(64)
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

            function abi_encode_tuple__to__fromStack(headStart ) -> tail {
                tail := add(headStart, 0)

            }

            function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
                revert(0, 0)
            }

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
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

            function abi_encode_t_uint256_to_t_uint256_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint256(value))
            }

            function abi_encode_tuple_t_uint256__to_t_uint256__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_uint256_to_t_uint256_fromStack(value0,  add(headStart, 0))

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
            /// @src 0:187:226  "mapping (address => uint) public credit"
            function getter_fun_credit_5(key_0) -> ret {

                let slot := 0
                let offset := 0

                slot := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(slot, key_0)

                ret := read_from_storage_split_dynamic_t_uint256(slot, offset)

            }
            /// @src 0:162:618  "contract SimpleDAO {..."

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

            /// @ast-id 18
            /// @src 0:233:316  "function donate(address to) public payable {..."
            function fun_donate_18(var_to_7) {

                /// @src 0:300:309  "msg.value"
                let expr_14 := callvalue()
                /// @src 0:286:292  "credit"
                let _1 := 0x00
                let expr_10 := _1
                /// @src 0:293:295  "to"
                let _2 := var_to_7
                let expr_11 := _2
                /// @src 0:286:296  "credit[to]"
                let _3 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_10,expr_11)
                /// @src 0:286:309  "credit[to] += msg.value"
                let _4 := read_from_storage_split_offset_0_t_uint256(_3)
                let expr_15 := checked_add_t_uint256(_4, expr_14)

                update_storage_value_offset_0t_uint256_to_t_uint256(_3, expr_15)

            }
            /// @src 0:162:618  "contract SimpleDAO {..."

            function array_storeLengthForEncoding_t_bytes_memory_ptr_nonPadded_inplace_fromStack(pos, length) -> updated_pos {
                updated_pos := pos
            }

            function store_literal_in_memory_c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470(memPtr) {

            }

            function abi_encode_t_stringliteral_c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470_to_t_bytes_memory_ptr_nonPadded_inplace_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_bytes_memory_ptr_nonPadded_inplace_fromStack(pos, 0)
                store_literal_in_memory_c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470(pos)
                end := add(pos, 0)
            }

            function abi_encode_tuple_packed_t_stringliteral_c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470__to_t_bytes_memory_ptr__nonPadded_inplace_fromStack(pos ) -> end {

                pos := abi_encode_t_stringliteral_c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470_to_t_bytes_memory_ptr_nonPadded_inplace_fromStack( pos)

                end := pos
            }

            function round_up_to_mul_of_32(value) -> result {
                result := and(add(value, 31), not(31))
            }

            function panic_error_0x41() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x41)
                revert(0, 0x24)
            }

            function finalize_allocation(memPtr, size) {
                let newFreePtr := add(memPtr, round_up_to_mul_of_32(size))
                // protect against overflow
                if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error_0x41() }
                mstore(64, newFreePtr)
            }

            function allocate_memory(size) -> memPtr {
                memPtr := allocate_unbounded()
                finalize_allocation(memPtr, size)
            }

            function array_allocation_size_t_bytes_memory_ptr(length) -> size {
                // Make sure we can allocate memory without overflow
                if gt(length, 0xffffffffffffffff) { panic_error_0x41() }

                size := round_up_to_mul_of_32(length)

                // add length slot
                size := add(size, 0x20)

            }

            function allocate_memory_array_t_bytes_memory_ptr(length) -> memPtr {
                let allocSize := array_allocation_size_t_bytes_memory_ptr(length)
                memPtr := allocate_memory(allocSize)

                mstore(memPtr, length)

            }

            function zero_value_for_split_t_bytes_memory_ptr() -> ret {
                ret := 96
            }

            function extract_returndata() -> data {

                switch returndatasize()
                case 0 {
                    data := zero_value_for_split_t_bytes_memory_ptr()
                }
                default {
                    data := allocate_memory_array_t_bytes_memory_ptr(returndatasize())
                    returndatacopy(add(data, 0x20), 0, returndatasize())
                }

            }

            function checked_sub_t_uint256(x, y) -> diff {
                x := cleanup_t_uint256(x)
                y := cleanup_t_uint256(y)

                if lt(x, y) { panic_error_0x11() }

                diff := sub(x, y)
            }

            /// @ast-id 49
            /// @src 0:322:522  "function withdraw(uint amount) public {..."
            function fun_withdraw_49(var_amount_20) {

                /// @src 0:374:380  "credit"
                let _5 := 0x00
                let expr_23 := _5
                /// @src 0:381:391  "msg.sender"
                let expr_25 := caller()
                /// @src 0:374:392  "credit[msg.sender]"
                let _6 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_23,expr_25)
                let _7 := read_from_storage_split_offset_0_t_uint256(_6)
                let expr_26 := _7
                /// @src 0:395:401  "amount"
                let _8 := var_amount_20
                let expr_27 := _8
                /// @src 0:374:401  "credit[msg.sender]>= amount"
                let expr_28 := iszero(lt(cleanup_t_uint256(expr_26), cleanup_t_uint256(expr_27)))
                /// @src 0:370:516  "if (credit[msg.sender]>= amount) {..."
                if expr_28 {
                    /// @src 0:432:442  "msg.sender"
                    let expr_32 := caller()
                    /// @src 0:432:447  "msg.sender.call"
                    let expr_33_address := expr_32
                    /// @src 0:454:460  "amount"
                    let _9 := var_amount_20
                    let expr_34 := _9
                    /// @src 0:432:461  "msg.sender.call{value:amount}"
                    let expr_35_address := expr_33_address
                    let expr_35_value := expr_34
                    /// @src 0:432:465  "msg.sender.call{value:amount}(\"\")"

                    let _10 := allocate_unbounded()
                    let _11 := sub(abi_encode_tuple_packed_t_stringliteral_c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470__to_t_bytes_memory_ptr__nonPadded_inplace_fromStack(_10  ), _10)

                    let expr_37_component_1 := call(gas(), expr_35_address,  expr_35_value,  _10, _11, 0, 0)
                    let expr_37_component_2_mpos := extract_returndata()
                    /// @src 0:417:465  "(bool res, ) = msg.sender.call{value:amount}(\"\")"
                    let var_res_30 := expr_37_component_1
                    /// @src 0:499:505  "amount"
                    let _12 := var_amount_20
                    let expr_43 := _12
                    /// @src 0:479:485  "credit"
                    let _13 := 0x00
                    let expr_39 := _13
                    /// @src 0:486:496  "msg.sender"
                    let expr_41 := caller()
                    /// @src 0:479:497  "credit[msg.sender]"
                    let _14 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_39,expr_41)
                    /// @src 0:479:505  "credit[msg.sender]-=amount"
                    let _15 := read_from_storage_split_offset_0_t_uint256(_14)
                    let expr_44 := checked_sub_t_uint256(_15, expr_43)

                    update_storage_value_offset_0t_uint256_to_t_uint256(_14, expr_44)
                    /// @src 0:370:516  "if (credit[msg.sender]>= amount) {..."
                }

            }
            /// @src 0:162:618  "contract SimpleDAO {..."

            function zero_value_for_split_t_uint256() -> ret {
                ret := 0
            }

            /// @ast-id 61
            /// @src 0:528:616  "function queryCredit(address to) public returns (uint){..."
            function fun_queryCredit_61(var_to_51) -> var__54 {
                /// @src 0:577:581  "uint"
                let zero_t_uint256_16 := zero_value_for_split_t_uint256()
                var__54 := zero_t_uint256_16

                /// @src 0:599:605  "credit"
                let _17 := 0x00
                let expr_56 := _17
                /// @src 0:606:608  "to"
                let _18 := var_to_51
                let expr_57 := _18
                /// @src 0:599:609  "credit[to]"
                let _19 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_56,expr_57)
                let _20 := read_from_storage_split_offset_0_t_uint256(_19)
                let expr_58 := _20
                /// @src 0:592:609  "return credit[to]"
                var__54 := expr_58
                leave

            }
            /// @src 0:162:618  "contract SimpleDAO {..."

        }

        data ".metadata" hex"a36469706673582212207694b118df7ff88d659d707f4fa9486e60ed17d41e5a6e2041b5a34e91be10ad6c6578706572696d656e74616cf564736f6c63430008090041"
    }
}

