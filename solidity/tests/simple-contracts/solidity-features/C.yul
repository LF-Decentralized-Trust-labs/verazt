/*******************************************************
 *                       WARNING                       *
 *  Solidity to Yul compilation is still EXPERIMENTAL  *
 *       It can result in LOSS OF FUNDS or worse       *
 *                !USE AT YOUR OWN RISK!               *
 *******************************************************/


object "C_55" {
    code {
        mstore(64, 128)
        if callvalue() { revert(0, 0) }

        constructor_C_55()

        codecopy(0, dataoffset("C_55_deployed"), datasize("C_55_deployed"))

        return(0, datasize("C_55_deployed"))

        function constructor_C_55() {

        }

    }
    object "C_55_deployed" {
        code {
            mstore(64, 128)

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0xf8a8fd6d
                {
                    // test()
                    if callvalue() { revert(0, 0) }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0 :=  fun_test_54()
                    let memPos := allocateMemory(0)
                    let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                default {}
            }
            if iszero(calldatasize()) {  }
            revert(0, 0)

            function abi_decode_tuple_(headStart, dataEnd)   {
                if slt(sub(dataEnd, headStart), 0) { revert(0, 0) }

            }

            function abi_encode_t_uint256_to_t_uint256_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint256(value))
            }

            function abi_encode_tuple_t_uint256__to_t_uint256__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_uint256_to_t_uint256_fromStack(value0,  add(headStart, 0))

            }

            function allocateMemory(size) -> memPtr {
                memPtr := mload(64)
                let newFreePtr := add(memPtr, size)
                // protect against overflow
                if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error() }
                mstore(64, newFreePtr)
            }

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function convert_t_rational_0_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(value)
            }

            function convert_t_rational_1_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(value)
            }

            function convert_t_rational_2_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(value)
            }

            function convert_t_rational_57896044618658097711785492504343953926634992332820282019728792003956564819968_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(value)
            }

            function convert_t_rational_7_by_1_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(value)
            }

            function fun_test_54() -> vloc__3 {
                let zero_value_for_type_t_uint256_1 := zero_value_for_split_t_uint256()
                vloc__3 := zero_value_for_type_t_uint256_1

                let expr_14 := 0x02
                let expr_18 := 0x8000000000000000000000000000000000000000000000000000000000000000
                let expr_21 := 0x8000000000000000000000000000000000000000000000000000000000000000
                let expr_22 := 0x07
                let _2 := convert_t_rational_7_by_1_to_t_uint256(expr_22)
                if iszero(_2) { panic_error() }
                let expr_23 := addmod(convert_t_rational_57896044618658097711785492504343953926634992332820282019728792003956564819968_by_1_to_t_uint256(expr_18), convert_t_rational_57896044618658097711785492504343953926634992332820282019728792003956564819968_by_1_to_t_uint256(expr_21), _2)
                let expr_24 := iszero(eq(convert_t_rational_2_by_1_to_t_uint256(expr_14), cleanup_t_uint256(expr_23)))
                if expr_24 {
                    let expr_25 := 0x01
                    vloc__3 := convert_t_rational_1_by_1_to_t_uint256(expr_25)
                    leave
                }
                let expr_37 := 0x02
                let expr_41 := 0x8000000000000000000000000000000000000000000000000000000000000000
                let expr_44 := 0x8000000000000000000000000000000000000000000000000000000000000000
                let expr_45 := 0x07
                let _3 := convert_t_rational_7_by_1_to_t_uint256(expr_45)
                if iszero(_3) { panic_error() }
                let expr_46 := addmod(convert_t_rational_57896044618658097711785492504343953926634992332820282019728792003956564819968_by_1_to_t_uint256(expr_41), convert_t_rational_57896044618658097711785492504343953926634992332820282019728792003956564819968_by_1_to_t_uint256(expr_44), _3)
                let expr_47 := iszero(eq(convert_t_rational_2_by_1_to_t_uint256(expr_37), cleanup_t_uint256(expr_46)))
                if expr_47 {
                    let expr_48 := 0x02
                    vloc__3 := convert_t_rational_2_by_1_to_t_uint256(expr_48)
                    leave
                }
                let expr_51 := 0x00
                vloc__3 := convert_t_rational_0_by_1_to_t_uint256(expr_51)
                leave

            }

            function panic_error() {
                invalid()
            }

            function shift_right_224_unsigned(value) -> newValue {
                newValue :=

                shr(224, value)

            }

            function zero_value_for_split_t_uint256() -> ret {
                ret := 0
            }

        }

    }

}

