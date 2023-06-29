#pragma once

#include "new_fp_u256.h.metal"

#include "../fft/fft.h.metal"
#include "../fft/twiddles.h.metal"
#include "../fft/permutation.h.metal"

// Prime Field of U256 with modulus 0x800000000000011000000000000000000000000000000000000000000000001, used for Starks
namespace {
  typedef NewFp256 Fp;
}

template [[ host_name("radix2_dit_butterfly_stark256") ]] 
[[kernel]] void radix2_dit_butterfly<Fp>(
    device Fp*, 
    constant Fp*, 
    uint32_t, 
    uint32_t, 
    uint32_t
);

template [[ host_name("calc_twiddles_stark256") ]] 
[[kernel]] void calc_twiddles<Fp>(
    device Fp*,
    constant Fp&, 
    uint
);

template [[ host_name("calc_twiddles_inv_stark256") ]] 
[[kernel]] void calc_twiddles_inv<Fp>(
    device Fp*,
    constant Fp&, 
    uint
);

template [[ host_name("calc_twiddles_bitrev_stark256") ]] 
[[kernel]] void calc_twiddles_bitrev<Fp>(
    device Fp*,
    constant Fp&, 
    uint,
    uint
);

template [[ host_name("calc_twiddles_bitrev_inv_stark256") ]] 
[[kernel]] void calc_twiddles_bitrev_inv<Fp>(
    device Fp*,
    constant Fp&, 
    uint,
    uint
);

template [[ host_name("bitrev_permutation_stark256") ]] 
[[kernel]] void bitrev_permutation<Fp>(
    device Fp*, 
    device Fp*, 
    uint, 
    uint
);
