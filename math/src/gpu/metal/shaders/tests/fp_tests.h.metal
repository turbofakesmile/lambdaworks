#pragma once

namespace fp_tests {
    template<typename Fp>
    [[kernel]] void add(
        constant Fp &_p [[ buffer(0) ]],
        constant Fp &_q [[ buffer(1) ]],
        device Fp &result [[ buffer(2) ]]
    )
    {
        Fp p = _p;
        Fp q = _q;
        Fp res = p + q;
        result = res;
    }

    template<typename Fp>
    [[kernel]] void sub(
        constant Fp &_p [[ buffer(0) ]],
        constant Fp &_q [[ buffer(1) ]],
        device Fp &result [[ buffer(2) ]]
    )
    {
        Fp p = _p;
        Fp q = _q;
        Fp res = p - q;
        result = res;
    }

    template<typename Fp>
    [[kernel]] void mul(
        constant Fp &_p [[ buffer(0) ]],
        constant Fp &_q [[ buffer(1) ]],
        device Fp &result [[ buffer(2) ]]
    )
    {
        Fp p = _p;
        Fp q = _q;
        Fp res = p * q;
        result = res;
    }

    template<typename Fp>
    [[kernel]] void pow(
        constant Fp &_p [[ buffer(0) ]],
        constant uint32_t &_a [[ buffer(1) ]],
        device Fp &result [[ buffer(2) ]]
    )
    {
        Fp p = _p;
        uint32_t a = _a;
        Fp res = p.pow(a);
        result = res;
    }
}
