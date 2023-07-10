#pragma once

#include "../field/new_fp_u256.h.metal"
#include "fp_tests.h.metal"

namespace {
  typedef NewFp256 Fp;
}

template [[ host_name("test_add_stark256") ]] 
[[kernel]] void fp_tests::add(
    constant Fp &_p,
    constant Fp &_q,
    device Fp &result
);

template [[ host_name("test_sub_stark256") ]] 
[[kernel]] void fp_tests::sub(
    constant Fp &_p,
    constant Fp &_q,
    device Fp &result
);

template [[ host_name("test_mul_stark256") ]] 
[[kernel]] void fp_tests::mul(
    constant Fp &_p,
    constant Fp &_q,
    device Fp &result
);

template [[ host_name("test_pow_stark256") ]] 
[[kernel]] void fp_tests::pow(
    constant Fp &_p,
    constant uint32_t &_a,
    device Fp &result
);

template [[ host_name("test_inv_stark256") ]] 
[[kernel]] void fp_tests::inv(
    constant Fp &_p,
    device Fp &result
);
