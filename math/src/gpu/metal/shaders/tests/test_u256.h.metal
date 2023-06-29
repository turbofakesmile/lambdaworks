#pragma once

#include "../field/unsigned_int.h.metal"
#include "uint_tests.h.metal"

namespace {
  typedef UnsignedInteger<8> U;
}

template [[ host_name("test_add_u256") ]] 
[[kernel]] void uint_tests::add(
    constant U &_a,
    constant U &_b,
    device U &result
);

template [[ host_name("test_sub_u256") ]] 
[[kernel]] void uint_tests::sub(
    constant U &_a,
    constant U &_b,
    device U &result
);

template [[ host_name("test_mul_u256") ]] 
[[kernel]] void uint_tests::mul(
    constant U &_a,
    constant U &_b,
    device U &result
);

template [[ host_name("test_shl_u256") ]] 
[[kernel]] void uint_tests::shl(
    constant U &_a,
    constant uint64_t &_b,
    device U &result
);

template [[ host_name("test_shr_u256") ]] 
[[kernel]] void uint_tests::shr(
    constant U &_a,
    constant uint64_t &_b,
    device U &result
);
