#!/bin/bash


cd "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cargo clean

cargo build


# For vec_get.rs
# ./target/debug/mir-checker ./benchmark/index/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./vec_get_const_result.txt 2>&1

# For no_loop.rs
# ./target/debug/mir-checker ./benchmark/no_loop/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./no_loop_const_result.txt 2>&1

# For origin safebug out-of-bound
# ./target/debug/mir-checker tests/safe-bugs/out-of-bound-index/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./oob.result.txt 2>&1

# For costum_func
# ./target/debug/mir-checker ./benchmark/costum_func/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./benchmark/costum_func/costum_func_result.txt 2>&1

# For lib_func
# ./target/debug/mir-checker ./benchmark/lib_func/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./benchmark/lib_func/lib_func_result.txt 2>&1

# For unsafe get_unchecked_mut test
# ./target/debug/mir-checker ./tests/unsafe-func-bugs/get_unchecked_mut/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./unsafe_get_unchecked_mut_result.txt 2>&1

# For unsafe from_raw_parts test
# ./target/debug/mir-checker ./tests/unsafe-func-bugs/from_raw_parts_mut/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./from_raw_parts_mut_result.txt 2>&1

# For from_raw_parts_in_loop
# ./target/debug/mir-checker ./benchmark/from_raw_parts_in_loop/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./benchmark/from_raw_parts_in_loop/from_raw_parts_in_loop_result.txt 2>&1

# For if-else 
# ./target/debug/mir-checker ./benchmark/if_else/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 > ./benchmark/if_else/if_else_result.txt 2>&1

# For new_known_name
# ./target/debug/mir-checker ./benchmark/new_known_name_demo/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5

# For tests

./target/debug/mir-checker ./tests/get/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5

