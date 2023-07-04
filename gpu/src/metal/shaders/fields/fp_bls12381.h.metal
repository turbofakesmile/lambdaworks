#pragma once

#include "unsigned_int.h.metal"


namespace {
    typedef UnsignedInteger<8> U256;
}

// taken from the Rust implementation
constexpr static const constant U256 N = {
    0x08000000,0x00000011,
    0x00000000,0x00000000,
    0x00000000,0x00000000,
    0x00000000,0x00000001
};
constexpr static const constant U256 R_SQUARED = {
    0x07FFD4AB,0x5E008810,
    0xFFFFFFFF,0xFF6F8000,
    0x00000001,0x330FFFFF,
    0xFFFFFD73,0x7E000401,
};

// Equates to `(1 << 256) - N`
constexpr static const constant U256 R_SUB_N = {
    0xf7ffffff, 0xffffffee,
    0xffffffff, 0xffffffff, 
    0xffffffff, 0xffffffff, 
    0xffffffff, 0xffffffff
};

// MU = -N^{-1} mod (2^32)
constexpr static const constant uint64_t MU = 4294967295;

class FpBLS12381 {
public:
    U256 inner;
    constexpr FpBLS12381() = default;
    constexpr FpBLS12381(uint64_t v) : inner{U256::from_int(v)} {}
    constexpr FpBLS12381(U256 v) : inner{v} {}

    constexpr explicit operator U256() const
    {
        return inner;
    }

    constexpr FpBLS12381 operator+(const FpBLS12381 rhs) const
    {
        return FpBLS12381(add(inner, rhs.inner));
    }

    constexpr FpBLS12381 operator-(const FpBLS12381 rhs) const
    {
        return FpBLS12381(sub(inner, rhs.inner));
    }

    constexpr FpBLS12381 operator*(const FpBLS12381 rhs) const
    {
        return FpBLS12381(mul(inner, rhs.inner));
    }

    constexpr bool operator==(const FpBLS12381 rhs) const
    {
        return inner == rhs.inner;
    }

    constexpr bool operator!=(const FpBLS12381 rhs) const
    {
        return !(inner == rhs.inner);
    }

    constexpr explicit operator uint32_t() const
    {
        return inner.m_limbs[7];
    }

    FpBLS12381 operator>>(const uint32_t rhs) const
    {
        return FpBLS12381(inner >> rhs);
    }

    FpBLS12381 operator<<(const uint32_t rhs) const
    {
        return FpBLS12381(inner << rhs);
    }

    constexpr static FpBLS12381 one()
    {
        // TODO find a way to generate on compile time
        const FpBLS12381 ONE = FpBLS12381::mul(U256::from_int((uint32_t) 1), R_SQUARED);
        return ONE;
    }

    constexpr FpBLS12381 to_montgomery()
    {
        return mul(inner, R_SQUARED);
    }

    // TODO: make method for all fields
    FpBLS12381 pow(uint32_t exp) const
    {
        // TODO find a way to generate on compile time
        FpBLS12381 const ONE = one();
        FpBLS12381 res = ONE;
        FpBLS12381 power = *this;

        while (exp > 0)
        {
            if (exp & 1)
            {
                res = res * power;
            }
            exp >>= 1;
            power = power * power;
        }

        return res;
    }

    FpBLS12381 inverse() 
    {
        // used addchain
        // https://github.com/mmcloughlin/addchain
        U256 _10 = mul(inner, inner);
        U256 _11 = mul(_10, inner);
        U256 _1100 = sqn<2>(_11);
        U256 _1101 = mul(inner, _1100);
        U256 _1111 = mul(_10, _1101);
        U256 _11001 = mul(_1100, _1101);
        U256 _110010 = mul(_11001, _11001);
        U256 _110011 = mul(inner, _110010);
        U256 _1000010 = mul(_1111, _110011);
        U256 _1001110 = mul(_1100, _1000010);
        U256 _10000001 = mul(_110011, _1001110);
        U256 _11001111 = mul(_1001110, _10000001);
        U256 i14 = mul(_11001111, _11001111);
        U256 i15 = mul(_10000001, i14);
        U256 i16 = mul(i14, i15);
        U256 x10 = mul(_1000010, i16);
        U256 i27 = sqn<10>(x10);
        U256 i28 = mul(i16, i27);
        U256 i38 = sqn<10>(i27);
        U256 i39 = mul(i28, i38);
        U256 i49 = sqn<10>(i38);
        U256 i50 = mul(i39, i49);
        U256 i60 = sqn<10>(i49);
        U256 i61 = mul(i50, i60);
        U256 i72 = mul(sqn<10>(i60), i61);
        U256 x60 = mul(_1000010, i72);
        U256 i76 = sqn<2>(mul(i72, x60));
        U256 x64 = mul(mul(i15, i76), i76);
        U256 i208 = mul(sqn<64>(mul(sqn<63>(mul(i15, x64)), x64)), x64);
        return FpBLS12381(mul(sqn<60>(i208), x60));
    }

    FpBLS12381 neg()
    {
        // TODO: can improve
        return FpBLS12381(sub(U256::from_int((uint32_t)0), inner));
    }

private:

    template<uint32_t N_ACC>
    U256 sqn(U256 base) const {
        U256 result = base;
#pragma unroll
        for (uint32_t i = 0; i < N_ACC; i++) {
            result = mul(result, result);
        }
        return result;
    }

    // Computes `lhs + rhs mod N`
    // Returns value in range [0,N)
    inline U256 add(const U256 lhs, const U256 rhs) const
    {
        U256 addition = lhs + rhs;
        U256 res = addition;
        // TODO: determine if an if statement here are more optimal

        return res - U256::from_int((uint64_t)(addition >= N)) * N + U256::from_int((uint64_t)(addition < lhs)) * R_SUB_N;
    }

    // Computes `lhs - rhs mod N`
    // Assumes `rhs` value in range [0,N)
    inline U256 sub(const U256 lhs, const U256 rhs) const
    {
        return add(lhs, ((U256)N) - rhs);
    }

    // Computes `lhs * rhs mod M`
    //
    // Essential that inputs are already in the range [0,N) and are in montgomery
    // form. Multiplication performs single round of montgomery reduction.
    //
    // Reference:
    // - https://en.wikipedia.org/wiki/Montgomery_modular_multiplication (REDC)
    // - https://www.youtube.com/watch?v=2UmQDKcelBQ
    constexpr static U256 mul(const U256 a, const U256 b)
    {
        constexpr uint64_t NUM_LIMBS = 8;
        metal::array<uint32_t, NUM_LIMBS> t = {};
        metal::array<uint32_t, 2> t_extra = {};

        U256 q = N;

        uint64_t i = NUM_LIMBS;

        while (i > 0) {
            i -= 1;
            // C := 0
            uint64_t c = 0;

            // for j=0 to N-1
            //    (C,t[j]) := t[j] + a[j]*b[i] + C
            uint64_t cs = 0;
            uint64_t j = NUM_LIMBS;
            while (j > 0) {
                j -= 1;
                cs = (uint64_t)t[j] + (uint64_t)a.m_limbs[j] * (uint64_t)b.m_limbs[i] + c;
                c = cs >> 32;
                t[j] = (uint32_t)cs;
            }

            // (t[N+1],t[N]) := t[N] + C
            cs = (uint64_t)t_extra[1] + c;
            t_extra[0] = (uint32_t)(cs >> 32);
            t_extra[1] = (uint32_t)cs;

            // m := t[0]*q'[0] mod D
            uint64_t m = ((uint64_t)t[NUM_LIMBS - 1] * MU) & 0xFFFFFFFF;

            // (C,_) := t[0] + m*q[0]
            c = ((uint64_t)t[NUM_LIMBS - 1] + m * (uint64_t)q.m_limbs[NUM_LIMBS - 1]) >> 32;

            // for j=1 to N-1
            //    (C,t[j-1]) := t[j] + m*q[j] + C

            j = NUM_LIMBS - 1;
            while (j > 0) {
                j -= 1;
                cs = (uint64_t)t[j] + m * (uint64_t)q.m_limbs[j] + c;
                c = cs >> 32;
                t[j + 1] = (uint32_t)cs;
            }

            // (C,t[N-1]) := t[N] + C
            cs = (uint64_t)t_extra[1] + c;
            c = cs >> 32;
            t[0] = (uint32_t)cs;

            // t[N] := t[N+1] + C
            t_extra[1] = t_extra[0] + (uint32_t)c;
        }

        U256 result {t};

        uint64_t overflow = t_extra[0] > 0;
        // TODO: assuming the integer represented by
        // [t_extra[1], t[0], ..., t[NUM_LIMBS - 1]] is at most
        // 2q in any case.
        if (overflow || q <= result) {
            result = result - q;
        }

        return result;
    }
};
