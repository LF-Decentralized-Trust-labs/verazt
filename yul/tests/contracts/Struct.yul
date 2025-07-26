/*=====================================================*
 *                       WARNING                       *
 *  Solidity to Yul compilation is still EXPERIMENTAL  *
 *       It can result in LOSS OF FUNDS or worse       *
 *                !USE AT YOUR OWN RISK!               *
 *=====================================================*/


/// @use-src 0:"Struct.sol"
object "Struct_72" {
    code {
        /// @src 0:110:1221  "contract Struct {..."
        mstore(64, memoryguard(128))
        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

        constructor_Struct_72()

        let _1 := allocate_unbounded()
        codecopy(_1, dataoffset("Struct_72_deployed"), datasize("Struct_72_deployed"))

        return(_1, datasize("Struct_72_deployed"))

        function allocate_unbounded() -> memPtr {
            memPtr := mload(64)
        }

        function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
            revert(0, 0)
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

        function allocate_memory_struct_t_struct$_Book_$10_storage_ptr() -> memPtr {
            memPtr := allocate_memory(128)
        }

        function array_allocation_size_t_string_memory_ptr(length) -> size {
            // Make sure we can allocate memory without overflow
            if gt(length, 0xffffffffffffffff) { panic_error_0x41() }

            size := round_up_to_mul_of_32(length)

            // add length slot
            size := add(size, 0x20)

        }

        function allocate_memory_array_t_string_memory_ptr(length) -> memPtr {
            let allocSize := array_allocation_size_t_string_memory_ptr(length)
            memPtr := allocate_memory(allocSize)

            mstore(memPtr, length)

        }

        function store_literal_in_memory_0a1be0ac746c1993f7d25afce258a1656d6d5da7335cfab8b16287a24c1e7e2f(memPtr) {

            mstore(add(memPtr, 0), "Building Ethereum DApps")

        }

        function copy_literal_to_memory_0a1be0ac746c1993f7d25afce258a1656d6d5da7335cfab8b16287a24c1e7e2f() -> memPtr {
            memPtr := allocate_memory_array_t_string_memory_ptr(23)
            store_literal_in_memory_0a1be0ac746c1993f7d25afce258a1656d6d5da7335cfab8b16287a24c1e7e2f(add(memPtr, 32))
        }

        function convert_t_stringliteral_0a1be0ac746c1993f7d25afce258a1656d6d5da7335cfab8b16287a24c1e7e2f_to_t_string_memory_ptr() -> converted {
            converted := copy_literal_to_memory_0a1be0ac746c1993f7d25afce258a1656d6d5da7335cfab8b16287a24c1e7e2f()
        }

        function write_to_memory_t_string_memory_ptr(memPtr, value) {
            mstore(memPtr, value)
        }

        function store_literal_in_memory_fb3088211d9100dcde654df64fb837cc44d9929bb1809872e710aecb2cf76bce(memPtr) {

            mstore(add(memPtr, 0), "Roberto Infante ")

        }

        function copy_literal_to_memory_fb3088211d9100dcde654df64fb837cc44d9929bb1809872e710aecb2cf76bce() -> memPtr {
            memPtr := allocate_memory_array_t_string_memory_ptr(16)
            store_literal_in_memory_fb3088211d9100dcde654df64fb837cc44d9929bb1809872e710aecb2cf76bce(add(memPtr, 32))
        }

        function convert_t_stringliteral_fb3088211d9100dcde654df64fb837cc44d9929bb1809872e710aecb2cf76bce_to_t_string_memory_ptr() -> converted {
            converted := copy_literal_to_memory_fb3088211d9100dcde654df64fb837cc44d9929bb1809872e710aecb2cf76bce()
        }

        function cleanup_t_rational_2_by_1(value) -> cleaned {
            cleaned := value
        }

        function cleanup_t_uint256(value) -> cleaned {
            cleaned := value
        }

        function identity(value) -> ret {
            ret := value
        }

        function convert_t_rational_2_by_1_to_t_uint256(value) -> converted {
            converted := cleanup_t_uint256(identity(cleanup_t_rational_2_by_1(value)))
        }

        function write_to_memory_t_uint256(memPtr, value) {
            mstore(memPtr, cleanup_t_uint256(value))
        }

        function cleanup_t_bool(value) -> cleaned {
            cleaned := iszero(iszero(value))
        }

        function write_to_memory_t_bool(memPtr, value) {
            mstore(memPtr, cleanup_t_bool(value))
        }

        function panic_error_0x00() {
            mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
            mstore(4, 0x00)
            revert(0, 0x24)
        }

        function read_from_memoryt_string_memory_ptr(memPtr) -> value {
            value := mload(memPtr)
        }

        function array_length_t_string_memory_ptr(value) -> length {

            length := mload(value)

        }

        function panic_error_0x22() {
            mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
            mstore(4, 0x22)
            revert(0, 0x24)
        }

        function extract_byte_array_length(data) -> length {
            length := div(data, 2)
            let outOfPlaceEncoding := and(data, 1)
            if iszero(outOfPlaceEncoding) {
                length := and(length, 0x7f)
            }

            if eq(outOfPlaceEncoding, lt(length, 32)) {
                panic_error_0x22()
            }
        }

        function array_dataslot_t_string_storage(ptr) -> data {
            data := ptr

            mstore(0, ptr)
            data := keccak256(0, 0x20)

        }

        function divide_by_32_ceil(value) -> result {
            result := div(add(value, 31), 32)
        }

        function shift_left_dynamic(bits, value) -> newValue {
            newValue :=

            shl(bits, value)

        }

        function update_byte_slice_dynamic32(value, shiftBytes, toInsert) -> result {
            let shiftBits := mul(shiftBytes, 8)
            let mask := shift_left_dynamic(shiftBits, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
            toInsert := shift_left_dynamic(shiftBits, toInsert)
            value := and(value, not(mask))
            result := or(value, and(toInsert, mask))
        }

        function convert_t_uint256_to_t_uint256(value) -> converted {
            converted := cleanup_t_uint256(identity(cleanup_t_uint256(value)))
        }

        function prepare_store_t_uint256(value) -> ret {
            ret := value
        }

        function update_storage_value_t_uint256_to_t_uint256(slot, offset, value_0) {
            let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
            sstore(slot, update_byte_slice_dynamic32(sload(slot), offset, prepare_store_t_uint256(convertedValue_0)))
        }

        function zero_value_for_split_t_uint256() -> ret {
            ret := 0
        }

        function storage_set_to_zero_t_uint256(slot, offset) {
            let zero_0 := zero_value_for_split_t_uint256()
            update_storage_value_t_uint256_to_t_uint256(slot, offset, zero_0)
        }

        function clear_storage_range_t_bytes1(start, end) {
            for {} lt(start, end) { start := add(start, 1) }
            {
                storage_set_to_zero_t_uint256(start, 0)
            }
        }

        function clean_up_bytearray_end_slots_t_string_storage(array, len, startIndex) {

            if gt(len, 31) {
                let dataArea := array_dataslot_t_string_storage(array)
                let deleteStart := add(dataArea, divide_by_32_ceil(startIndex))
                // If we are clearing array to be short byte array, we want to clear only data starting from array data area.
                if lt(startIndex, 32) { deleteStart := dataArea }
                clear_storage_range_t_bytes1(deleteStart, add(dataArea, divide_by_32_ceil(len)))
            }

        }

        function shift_right_unsigned_dynamic(bits, value) -> newValue {
            newValue :=

            shr(bits, value)

        }

        function mask_bytes_dynamic(data, bytes) -> result {
            let mask := not(shift_right_unsigned_dynamic(mul(8, bytes), not(0)))
            result := and(data, mask)
        }
        function extract_used_part_and_set_length_of_short_byte_array(data, len) -> used {
            // we want to save only elements that are part of the array after resizing
            // others should be set to zero
            data := mask_bytes_dynamic(data, len)
            used := or(data, mul(2, len))
        }
        function copy_byte_array_to_storage_from_t_string_memory_ptr_to_t_string_storage(slot, src) {

            let newLen := array_length_t_string_memory_ptr(src)
            // Make sure array length is sane
            if gt(newLen, 0xffffffffffffffff) { panic_error_0x41() }

            let oldLen := extract_byte_array_length(sload(slot))

            // potentially truncate data
            clean_up_bytearray_end_slots_t_string_storage(slot, oldLen, newLen)

            let srcOffset := 0

            srcOffset := 0x20

            switch gt(newLen, 31)
            case 1 {
                let loopEnd := and(newLen, not(0x1f))

                let dstPtr := array_dataslot_t_string_storage(slot)
                let i := 0
                for { } lt(i, loopEnd) { i := add(i, 0x20) } {
                    sstore(dstPtr, mload(add(src, srcOffset)))
                    dstPtr := add(dstPtr, 1)
                    srcOffset := add(srcOffset, 32)
                }
                if lt(loopEnd, newLen) {
                    let lastValue := mload(add(src, srcOffset))
                    sstore(dstPtr, mask_bytes_dynamic(lastValue, and(newLen, 0x1f)))
                }
                sstore(slot, add(mul(newLen, 2), 1))
            }
            default {
                let value := 0
                if newLen {
                    value := mload(add(src, srcOffset))
                }
                sstore(slot, extract_used_part_and_set_length_of_short_byte_array(value, newLen))
            }
        }

        function update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(slot, value_0) {

            copy_byte_array_to_storage_from_t_string_memory_ptr_to_t_string_storage(slot, value_0)
        }

        function read_from_memoryt_uint256(ptr) -> returnValue {

            let value := cleanup_t_uint256(mload(ptr))

            returnValue :=

            value

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

        function update_storage_value_offset_0t_uint256_to_t_uint256(slot, value_0) {
            let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
            sstore(slot, update_byte_slice_32_shift_0(sload(slot), prepare_store_t_uint256(convertedValue_0)))
        }

        function read_from_memoryt_bool(ptr) -> returnValue {

            let value := cleanup_t_bool(mload(ptr))

            returnValue :=

            value

        }

        function update_byte_slice_1_shift_0(value, toInsert) -> result {
            let mask := 255
            toInsert := shift_left_0(toInsert)
            value := and(value, not(mask))
            result := or(value, and(toInsert, mask))
        }

        function convert_t_bool_to_t_bool(value) -> converted {
            converted := cleanup_t_bool(value)
        }

        function prepare_store_t_bool(value) -> ret {
            ret := value
        }

        function update_storage_value_offset_0t_bool_to_t_bool(slot, value_0) {
            let convertedValue_0 := convert_t_bool_to_t_bool(value_0)
            sstore(slot, update_byte_slice_1_shift_0(sload(slot), prepare_store_t_bool(convertedValue_0)))
        }

        function copy_struct_to_storage_from_t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(slot, value) {

            {

                let memberSlot := add(slot, 0)
                let memberSrcPtr := add(value, 0)

                let memberValue_0 := read_from_memoryt_string_memory_ptr(memberSrcPtr)

                update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(memberSlot, memberValue_0)

            }

            {

                let memberSlot := add(slot, 1)
                let memberSrcPtr := add(value, 32)

                let memberValue_0 := read_from_memoryt_string_memory_ptr(memberSrcPtr)

                update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(memberSlot, memberValue_0)

            }

            {

                let memberSlot := add(slot, 2)
                let memberSrcPtr := add(value, 64)

                let memberValue_0 := read_from_memoryt_uint256(memberSrcPtr)

                update_storage_value_offset_0t_uint256_to_t_uint256(memberSlot, memberValue_0)

            }

            {

                let memberSlot := add(slot, 3)
                let memberSrcPtr := add(value, 96)

                let memberValue_0 := read_from_memoryt_bool(memberSrcPtr)

                update_storage_value_offset_0t_bool_to_t_bool(memberSlot, memberValue_0)

            }

        }

        function update_storage_value_offset_0t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(slot, value_0) {

            copy_struct_to_storage_from_t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(slot, value_0)
        }

        /// @src 0:110:1221  "contract Struct {..."
        function constructor_Struct_72() {

            /// @src 0:110:1221  "contract Struct {..."

            /// @src 0:487:488  "2"
            let expr_19 := 0x02
            /// @src 0:490:495  "false"
            let expr_20 := 0x00
            /// @src 0:393:496  "Book(\"Building Ethereum DApps\",..."
            let expr_21_mpos := allocate_memory_struct_t_struct$_Book_$10_storage_ptr()
            let _2_mpos := convert_t_stringliteral_0a1be0ac746c1993f7d25afce258a1656d6d5da7335cfab8b16287a24c1e7e2f_to_t_string_memory_ptr()
            write_to_memory_t_string_memory_ptr(add(expr_21_mpos, 0), _2_mpos)
            let _3_mpos := convert_t_stringliteral_fb3088211d9100dcde654df64fb837cc44d9929bb1809872e710aecb2cf76bce_to_t_string_memory_ptr()
            write_to_memory_t_string_memory_ptr(add(expr_21_mpos, 32), _3_mpos)
            let _4 := convert_t_rational_2_by_1_to_t_uint256(expr_19)
            write_to_memory_t_uint256(add(expr_21_mpos, 64), _4)
            write_to_memory_t_bool(add(expr_21_mpos, 96), expr_20)
            update_storage_value_offset_0t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(0x04, expr_21_mpos)

        }
        /// @src 0:110:1221  "contract Struct {..."

    }
    /// @use-src 0:"Struct.sol"
    object "Struct_72_deployed" {
        code {
            /// @src 0:110:1221  "contract Struct {..."
            mstore(64, memoryguard(128))

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0x3e9a2451
                {
                    // set_book_detail()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    fun_set_book_detail_35()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple__to__fromStack(memPos  )
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x8574ed0d
                {
                    // book_info()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0, ret_1, ret_2, ret_3 :=  fun_book_info_57()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_string_memory_ptr_t_string_memory_ptr_t_uint256_t_bool__to_t_string_memory_ptr_t_string_memory_ptr_t_uint256_t_bool__fromStack(memPos , ret_0, ret_1, ret_2, ret_3)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0xb87f86b7
                {
                    // get_details()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0, ret_1 :=  fun_get_details_71()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_string_memory_ptr_t_uint256__to_t_string_memory_ptr_t_uint256__fromStack(memPos , ret_0, ret_1)
                    return(memPos, sub(memEnd, memPos))
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

            function abi_decode_tuple_(headStart, dataEnd)   {
                if slt(sub(dataEnd, headStart), 0) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

            }

            function abi_encode_tuple__to__fromStack(headStart ) -> tail {
                tail := add(headStart, 0)

            }

            function array_length_t_string_memory_ptr(value) -> length {

                length := mload(value)

            }

            function array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, length) -> updated_pos {
                mstore(pos, length)
                updated_pos := add(pos, 0x20)
            }

            function copy_memory_to_memory(src, dst, length) {
                let i := 0
                for { } lt(i, length) { i := add(i, 32) }
                {
                    mstore(add(dst, i), mload(add(src, i)))
                }
                if gt(i, length)
                {
                    // clear end
                    mstore(add(dst, length), 0)
                }
            }

            function round_up_to_mul_of_32(value) -> result {
                result := and(add(value, 31), not(31))
            }

            function abi_encode_t_string_memory_ptr_to_t_string_memory_ptr_fromStack(value, pos) -> end {
                let length := array_length_t_string_memory_ptr(value)
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, length)
                copy_memory_to_memory(add(value, 0x20), pos, length)
                end := add(pos, round_up_to_mul_of_32(length))
            }

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function abi_encode_t_uint256_to_t_uint256_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint256(value))
            }

            function cleanup_t_bool(value) -> cleaned {
                cleaned := iszero(iszero(value))
            }

            function abi_encode_t_bool_to_t_bool_fromStack(value, pos) {
                mstore(pos, cleanup_t_bool(value))
            }

            function abi_encode_tuple_t_string_memory_ptr_t_string_memory_ptr_t_uint256_t_bool__to_t_string_memory_ptr_t_string_memory_ptr_t_uint256_t_bool__fromStack(headStart , value0, value1, value2, value3) -> tail {
                tail := add(headStart, 128)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_string_memory_ptr_to_t_string_memory_ptr_fromStack(value0,  tail)

                mstore(add(headStart, 32), sub(tail, headStart))
                tail := abi_encode_t_string_memory_ptr_to_t_string_memory_ptr_fromStack(value1,  tail)

                abi_encode_t_uint256_to_t_uint256_fromStack(value2,  add(headStart, 64))

                abi_encode_t_bool_to_t_bool_fromStack(value3,  add(headStart, 96))

            }

            function abi_encode_tuple_t_string_memory_ptr_t_uint256__to_t_string_memory_ptr_t_uint256__fromStack(headStart , value0, value1) -> tail {
                tail := add(headStart, 64)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_string_memory_ptr_to_t_string_memory_ptr_fromStack(value0,  tail)

                abi_encode_t_uint256_to_t_uint256_fromStack(value1,  add(headStart, 32))

            }

            function revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74() {
                revert(0, 0)
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

            function allocate_memory_struct_t_struct$_Book_$10_storage_ptr() -> memPtr {
                memPtr := allocate_memory(128)
            }

            function array_allocation_size_t_string_memory_ptr(length) -> size {
                // Make sure we can allocate memory without overflow
                if gt(length, 0xffffffffffffffff) { panic_error_0x41() }

                size := round_up_to_mul_of_32(length)

                // add length slot
                size := add(size, 0x20)

            }

            function allocate_memory_array_t_string_memory_ptr(length) -> memPtr {
                let allocSize := array_allocation_size_t_string_memory_ptr(length)
                memPtr := allocate_memory(allocSize)

                mstore(memPtr, length)

            }

            function store_literal_in_memory_12ce2645d1b9432607c533d6e5aa999042be21ed5fb00b93788dfd2ff199f838(memPtr) {

                mstore(add(memPtr, 0), "Introducing Ethereum and Solidit")

                mstore(add(memPtr, 32), "y")

            }

            function copy_literal_to_memory_12ce2645d1b9432607c533d6e5aa999042be21ed5fb00b93788dfd2ff199f838() -> memPtr {
                memPtr := allocate_memory_array_t_string_memory_ptr(33)
                store_literal_in_memory_12ce2645d1b9432607c533d6e5aa999042be21ed5fb00b93788dfd2ff199f838(add(memPtr, 32))
            }

            function convert_t_stringliteral_12ce2645d1b9432607c533d6e5aa999042be21ed5fb00b93788dfd2ff199f838_to_t_string_memory_ptr() -> converted {
                converted := copy_literal_to_memory_12ce2645d1b9432607c533d6e5aa999042be21ed5fb00b93788dfd2ff199f838()
            }

            function write_to_memory_t_string_memory_ptr(memPtr, value) {
                mstore(memPtr, value)
            }

            function store_literal_in_memory_b8685e9e0771ad2ecc5ff89a2ce17be12d2888d80e6330397b700008bcd6f85a(memPtr) {

                mstore(add(memPtr, 0), "Chris Dannen")

            }

            function copy_literal_to_memory_b8685e9e0771ad2ecc5ff89a2ce17be12d2888d80e6330397b700008bcd6f85a() -> memPtr {
                memPtr := allocate_memory_array_t_string_memory_ptr(12)
                store_literal_in_memory_b8685e9e0771ad2ecc5ff89a2ce17be12d2888d80e6330397b700008bcd6f85a(add(memPtr, 32))
            }

            function convert_t_stringliteral_b8685e9e0771ad2ecc5ff89a2ce17be12d2888d80e6330397b700008bcd6f85a_to_t_string_memory_ptr() -> converted {
                converted := copy_literal_to_memory_b8685e9e0771ad2ecc5ff89a2ce17be12d2888d80e6330397b700008bcd6f85a()
            }

            function cleanup_t_rational_1_by_1(value) -> cleaned {
                cleaned := value
            }

            function identity(value) -> ret {
                ret := value
            }

            function convert_t_rational_1_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_rational_1_by_1(value)))
            }

            function write_to_memory_t_uint256(memPtr, value) {
                mstore(memPtr, cleanup_t_uint256(value))
            }

            function write_to_memory_t_bool(memPtr, value) {
                mstore(memPtr, cleanup_t_bool(value))
            }

            function panic_error_0x00() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x00)
                revert(0, 0x24)
            }

            function read_from_memoryt_string_memory_ptr(memPtr) -> value {
                value := mload(memPtr)
            }

            function panic_error_0x22() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x22)
                revert(0, 0x24)
            }

            function extract_byte_array_length(data) -> length {
                length := div(data, 2)
                let outOfPlaceEncoding := and(data, 1)
                if iszero(outOfPlaceEncoding) {
                    length := and(length, 0x7f)
                }

                if eq(outOfPlaceEncoding, lt(length, 32)) {
                    panic_error_0x22()
                }
            }

            function array_dataslot_t_string_storage(ptr) -> data {
                data := ptr

                mstore(0, ptr)
                data := keccak256(0, 0x20)

            }

            function divide_by_32_ceil(value) -> result {
                result := div(add(value, 31), 32)
            }

            function shift_left_dynamic(bits, value) -> newValue {
                newValue :=

                shl(bits, value)

            }

            function update_byte_slice_dynamic32(value, shiftBytes, toInsert) -> result {
                let shiftBits := mul(shiftBytes, 8)
                let mask := shift_left_dynamic(shiftBits, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
                toInsert := shift_left_dynamic(shiftBits, toInsert)
                value := and(value, not(mask))
                result := or(value, and(toInsert, mask))
            }

            function convert_t_uint256_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_uint256(value)))
            }

            function prepare_store_t_uint256(value) -> ret {
                ret := value
            }

            function update_storage_value_t_uint256_to_t_uint256(slot, offset, value_0) {
                let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
                sstore(slot, update_byte_slice_dynamic32(sload(slot), offset, prepare_store_t_uint256(convertedValue_0)))
            }

            function zero_value_for_split_t_uint256() -> ret {
                ret := 0
            }

            function storage_set_to_zero_t_uint256(slot, offset) {
                let zero_0 := zero_value_for_split_t_uint256()
                update_storage_value_t_uint256_to_t_uint256(slot, offset, zero_0)
            }

            function clear_storage_range_t_bytes1(start, end) {
                for {} lt(start, end) { start := add(start, 1) }
                {
                    storage_set_to_zero_t_uint256(start, 0)
                }
            }

            function clean_up_bytearray_end_slots_t_string_storage(array, len, startIndex) {

                if gt(len, 31) {
                    let dataArea := array_dataslot_t_string_storage(array)
                    let deleteStart := add(dataArea, divide_by_32_ceil(startIndex))
                    // If we are clearing array to be short byte array, we want to clear only data starting from array data area.
                    if lt(startIndex, 32) { deleteStart := dataArea }
                    clear_storage_range_t_bytes1(deleteStart, add(dataArea, divide_by_32_ceil(len)))
                }

            }

            function shift_right_unsigned_dynamic(bits, value) -> newValue {
                newValue :=

                shr(bits, value)

            }

            function mask_bytes_dynamic(data, bytes) -> result {
                let mask := not(shift_right_unsigned_dynamic(mul(8, bytes), not(0)))
                result := and(data, mask)
            }
            function extract_used_part_and_set_length_of_short_byte_array(data, len) -> used {
                // we want to save only elements that are part of the array after resizing
                // others should be set to zero
                data := mask_bytes_dynamic(data, len)
                used := or(data, mul(2, len))
            }
            function copy_byte_array_to_storage_from_t_string_memory_ptr_to_t_string_storage(slot, src) {

                let newLen := array_length_t_string_memory_ptr(src)
                // Make sure array length is sane
                if gt(newLen, 0xffffffffffffffff) { panic_error_0x41() }

                let oldLen := extract_byte_array_length(sload(slot))

                // potentially truncate data
                clean_up_bytearray_end_slots_t_string_storage(slot, oldLen, newLen)

                let srcOffset := 0

                srcOffset := 0x20

                switch gt(newLen, 31)
                case 1 {
                    let loopEnd := and(newLen, not(0x1f))

                    let dstPtr := array_dataslot_t_string_storage(slot)
                    let i := 0
                    for { } lt(i, loopEnd) { i := add(i, 0x20) } {
                        sstore(dstPtr, mload(add(src, srcOffset)))
                        dstPtr := add(dstPtr, 1)
                        srcOffset := add(srcOffset, 32)
                    }
                    if lt(loopEnd, newLen) {
                        let lastValue := mload(add(src, srcOffset))
                        sstore(dstPtr, mask_bytes_dynamic(lastValue, and(newLen, 0x1f)))
                    }
                    sstore(slot, add(mul(newLen, 2), 1))
                }
                default {
                    let value := 0
                    if newLen {
                        value := mload(add(src, srcOffset))
                    }
                    sstore(slot, extract_used_part_and_set_length_of_short_byte_array(value, newLen))
                }
            }

            function update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(slot, value_0) {

                copy_byte_array_to_storage_from_t_string_memory_ptr_to_t_string_storage(slot, value_0)
            }

            function read_from_memoryt_uint256(ptr) -> returnValue {

                let value := cleanup_t_uint256(mload(ptr))

                returnValue :=

                value

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

            function update_storage_value_offset_0t_uint256_to_t_uint256(slot, value_0) {
                let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
                sstore(slot, update_byte_slice_32_shift_0(sload(slot), prepare_store_t_uint256(convertedValue_0)))
            }

            function read_from_memoryt_bool(ptr) -> returnValue {

                let value := cleanup_t_bool(mload(ptr))

                returnValue :=

                value

            }

            function update_byte_slice_1_shift_0(value, toInsert) -> result {
                let mask := 255
                toInsert := shift_left_0(toInsert)
                value := and(value, not(mask))
                result := or(value, and(toInsert, mask))
            }

            function convert_t_bool_to_t_bool(value) -> converted {
                converted := cleanup_t_bool(value)
            }

            function prepare_store_t_bool(value) -> ret {
                ret := value
            }

            function update_storage_value_offset_0t_bool_to_t_bool(slot, value_0) {
                let convertedValue_0 := convert_t_bool_to_t_bool(value_0)
                sstore(slot, update_byte_slice_1_shift_0(sload(slot), prepare_store_t_bool(convertedValue_0)))
            }

            function copy_struct_to_storage_from_t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(slot, value) {

                {

                    let memberSlot := add(slot, 0)
                    let memberSrcPtr := add(value, 0)

                    let memberValue_0 := read_from_memoryt_string_memory_ptr(memberSrcPtr)

                    update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(memberSlot, memberValue_0)

                }

                {

                    let memberSlot := add(slot, 1)
                    let memberSrcPtr := add(value, 32)

                    let memberValue_0 := read_from_memoryt_string_memory_ptr(memberSrcPtr)

                    update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(memberSlot, memberValue_0)

                }

                {

                    let memberSlot := add(slot, 2)
                    let memberSrcPtr := add(value, 64)

                    let memberValue_0 := read_from_memoryt_uint256(memberSrcPtr)

                    update_storage_value_offset_0t_uint256_to_t_uint256(memberSlot, memberValue_0)

                }

                {

                    let memberSlot := add(slot, 3)
                    let memberSrcPtr := add(value, 96)

                    let memberValue_0 := read_from_memoryt_bool(memberSrcPtr)

                    update_storage_value_offset_0t_bool_to_t_bool(memberSlot, memberValue_0)

                }

            }

            function update_storage_value_offset_0t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(slot, value_0) {

                copy_struct_to_storage_from_t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(slot, value_0)
            }

            /// @ast-id 35
            /// @src 0:683:824  "function set_book_detail() public {..."
            function fun_set_book_detail_35() {

                /// @src 0:810:811  "1"
                let expr_29 := 0x01
                /// @src 0:813:817  "true"
                let expr_30 := 0x01
                /// @src 0:733:818  "Book(\"Introducing Ethereum and Solidity\",..."
                let expr_31_mpos := allocate_memory_struct_t_struct$_Book_$10_storage_ptr()
                let _1_mpos := convert_t_stringliteral_12ce2645d1b9432607c533d6e5aa999042be21ed5fb00b93788dfd2ff199f838_to_t_string_memory_ptr()
                write_to_memory_t_string_memory_ptr(add(expr_31_mpos, 0), _1_mpos)
                let _2_mpos := convert_t_stringliteral_b8685e9e0771ad2ecc5ff89a2ce17be12d2888d80e6330397b700008bcd6f85a_to_t_string_memory_ptr()
                write_to_memory_t_string_memory_ptr(add(expr_31_mpos, 32), _2_mpos)
                let _3 := convert_t_rational_1_by_1_to_t_uint256(expr_29)
                write_to_memory_t_uint256(add(expr_31_mpos, 64), _3)
                write_to_memory_t_bool(add(expr_31_mpos, 96), expr_30)
                /// @src 0:725:818  "book1 = Book(\"Introducing Ethereum and Solidity\",..."
                update_storage_value_offset_0t_struct$_Book_$10_memory_ptr_to_t_struct$_Book_$10_storage(0x00, expr_31_mpos)
                let _4_slot := 0x00
                let expr_32_slot := _4_slot

            }
            /// @src 0:110:1221  "contract Struct {..."

            function zero_value_for_split_t_string_memory_ptr() -> ret {
                ret := 96
            }

            function zero_value_for_split_t_bool() -> ret {
                ret := 0
            }

            function shift_right_0_unsigned(value) -> newValue {
                newValue :=

                shr(0, value)

            }

            function cleanup_from_storage_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function extract_from_storage_value_offset_0t_uint256(slot_value) -> value {
                value := cleanup_from_storage_t_uint256(shift_right_0_unsigned(slot_value))
            }

            function read_from_storage_split_offset_0_t_uint256(slot) -> value {
                value := extract_from_storage_value_offset_0t_uint256(sload(slot))

            }

            function cleanup_from_storage_t_bool(value) -> cleaned {
                cleaned := and(value, 0xff)
            }

            function extract_from_storage_value_offset_0t_bool(slot_value) -> value {
                value := cleanup_from_storage_t_bool(shift_right_0_unsigned(slot_value))
            }

            function read_from_storage_split_offset_0_t_bool(slot) -> value {
                value := extract_from_storage_value_offset_0t_bool(sload(slot))

            }

            function array_storeLengthForEncoding_t_string_memory_ptr(pos, length) -> updated_pos {
                mstore(pos, length)
                updated_pos := add(pos, 0x20)
            }

            // string -> string
            function abi_encode_t_string_storage_to_t_string_memory_ptr(value, pos) -> ret {
                let slotValue := sload(value)
                let length := extract_byte_array_length(slotValue)
                pos := array_storeLengthForEncoding_t_string_memory_ptr(pos, length)
                switch and(slotValue, 1)
                case 0 {
                    // short byte array
                    mstore(pos, and(slotValue, not(0xff)))
                    ret := add(pos, 0x20)
                }
                case 1 {
                    // long byte array
                    let dataPos := array_dataslot_t_string_storage(value)
                    let i := 0
                    for { } lt(i, length) { i := add(i, 0x20) } {
                        mstore(add(pos, i), sload(dataPos))
                        dataPos := add(dataPos, 1)
                    }
                    ret := add(pos, i)
                }
            }

            function abi_encodeUpdatedPos_t_string_storage_to_t_string_memory_ptr(value0, pos) -> updatedPos {
                updatedPos := abi_encode_t_string_storage_to_t_string_memory_ptr(value0, pos)
            }

            function copy_array_from_storage_to_memory_t_string_storage(slot) -> memPtr {
                memPtr := allocate_unbounded()
                let end := abi_encodeUpdatedPos_t_string_storage_to_t_string_memory_ptr(slot, memPtr)
                finalize_allocation(memPtr, sub(end, memPtr))
            }

            function convert_array_t_string_storage_to_t_string_memory_ptr(value) -> converted  {

                // Copy the array to a free position in memory
                converted :=

                copy_array_from_storage_to_memory_t_string_storage(value)

            }

            /// @ast-id 57
            /// @src 0:877:1059  "function book_info()public view returns (..."
            function fun_book_info_57() -> var__38_mpos, var__40_mpos, var__42, var__44 {
                /// @src 0:924:937  "string memory"
                let zero_t_string_memory_ptr_5_mpos := zero_value_for_split_t_string_memory_ptr()
                var__38_mpos := zero_t_string_memory_ptr_5_mpos
                /// @src 0:939:952  "string memory"
                let zero_t_string_memory_ptr_6_mpos := zero_value_for_split_t_string_memory_ptr()
                var__40_mpos := zero_t_string_memory_ptr_6_mpos
                /// @src 0:954:958  "uint"
                let zero_t_uint256_7 := zero_value_for_split_t_uint256()
                var__42 := zero_t_uint256_7
                /// @src 0:960:964  "bool"
                let zero_t_bool_8 := zero_value_for_split_t_bool()
                var__44 := zero_t_bool_8

                /// @src 0:984:989  "book2"
                let _9_slot := 0x04
                let expr_46_slot := _9_slot
                /// @src 0:984:994  "book2.name"
                let _10 := add(expr_46_slot, 0)
                let _11_slot := _10
                let expr_47_slot := _11_slot
                /// @src 0:983:1052  "(book2.name, book2.writter,..."
                let expr_54_component_1_slot := expr_47_slot
                /// @src 0:996:1001  "book2"
                let _12_slot := 0x04
                let expr_48_slot := _12_slot
                /// @src 0:996:1009  "book2.writter"
                let _13 := add(expr_48_slot, 1)
                let _14_slot := _13
                let expr_49_slot := _14_slot
                /// @src 0:983:1052  "(book2.name, book2.writter,..."
                let expr_54_component_2_slot := expr_49_slot
                /// @src 0:1026:1031  "book2"
                let _15_slot := 0x04
                let expr_50_slot := _15_slot
                /// @src 0:1026:1034  "book2.id"
                let _16 := add(expr_50_slot, 2)
                let _17 := read_from_storage_split_offset_0_t_uint256(_16)
                let expr_51 := _17
                /// @src 0:983:1052  "(book2.name, book2.writter,..."
                let expr_54_component_3 := expr_51
                /// @src 0:1036:1041  "book2"
                let _18_slot := 0x04
                let expr_52_slot := _18_slot
                /// @src 0:1036:1051  "book2.available"
                let _19 := add(expr_52_slot, 3)
                let _20 := read_from_storage_split_offset_0_t_bool(_19)
                let expr_53 := _20
                /// @src 0:983:1052  "(book2.name, book2.writter,..."
                let expr_54_component_4 := expr_53
                /// @src 0:977:1052  "return(book2.name, book2.writter,..."
                var__38_mpos := convert_array_t_string_storage_to_t_string_memory_ptr(expr_54_component_1_slot)
                var__40_mpos := convert_array_t_string_storage_to_t_string_memory_ptr(expr_54_component_2_slot)
                var__42 := expr_54_component_3
                var__44 := expr_54_component_4
                leave

            }
            /// @src 0:110:1221  "contract Struct {..."

            /// @ast-id 71
            /// @src 0:1111:1219  "function get_details() public view returns (string memory, uint) {..."
            function fun_get_details_71() -> var__60_mpos, var__62 {
                /// @src 0:1155:1168  "string memory"
                let zero_t_string_memory_ptr_21_mpos := zero_value_for_split_t_string_memory_ptr()
                var__60_mpos := zero_t_string_memory_ptr_21_mpos
                /// @src 0:1170:1174  "uint"
                let zero_t_uint256_22 := zero_value_for_split_t_uint256()
                var__62 := zero_t_uint256_22

                /// @src 0:1192:1197  "book1"
                let _23_slot := 0x00
                let expr_64_slot := _23_slot
                /// @src 0:1192:1202  "book1.name"
                let _24 := add(expr_64_slot, 0)
                let _25_slot := _24
                let expr_65_slot := _25_slot
                /// @src 0:1191:1213  "(book1.name, book1.id)"
                let expr_68_component_1_slot := expr_65_slot
                /// @src 0:1204:1209  "book1"
                let _26_slot := 0x00
                let expr_66_slot := _26_slot
                /// @src 0:1204:1212  "book1.id"
                let _27 := add(expr_66_slot, 2)
                let _28 := read_from_storage_split_offset_0_t_uint256(_27)
                let expr_67 := _28
                /// @src 0:1191:1213  "(book1.name, book1.id)"
                let expr_68_component_2 := expr_67
                /// @src 0:1184:1213  "return (book1.name, book1.id)"
                var__60_mpos := convert_array_t_string_storage_to_t_string_memory_ptr(expr_68_component_1_slot)
                var__62 := expr_68_component_2
                leave

            }
            /// @src 0:110:1221  "contract Struct {..."

        }

        data ".metadata" hex"a36469706673582212209ea7b5f3eb07a149bf812f0df16f4bfe5eba7d91a3a2a6ff4f838608a6306ba46c6578706572696d656e74616cf564736f6c634300080b0041"
    }

}

