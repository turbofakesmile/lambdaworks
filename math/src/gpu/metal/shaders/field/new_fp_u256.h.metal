#pragma once

#include "unsigned_int.h.metal"

namespace {
    typedef UnsignedInteger<12> U384;
}

// taken from the Rust implementation
constexpr static const constant U384 N = {
    0,0,
    0,0,
    0x08000000,0x00000011,
    0x00000000,0x00000000,
    0x00000000,0x00000000,
    0x00000000,0x00000001
};
constexpr static const constant U384 R_SQUARED = {
    0x11988fe5,0x92cae3aa,
    0x9a793e85,0xb519952d,
    0x67eb88a9,0x939d83c0,
    0x8de5476c,0x4c95b6d5,
    0x0a76e6a6,0x09d104f1,
    0xf4df1f34,0x1c341746
};

// Equates to `(1 << 384) - N`
constexpr static const constant U384 R_SUB_N = {
    0xe5feee15,0xc6801965,
    0xb4e45849,0xbcb45328,
    0x9b88b47b,0x0c7aed40,
    0x98cf2d5f,0x094f09db,
    0xe1540001,0x4eac0000,
    0x46010000,0x00005555
};

// MU = -N^{-1} mod (2^32)
constexpr static const constant uint64_t MU = 4294967295;

class NewFp256 {
public:
    U384 inner;
    constexpr NewFp256() = default;
    constexpr NewFp256(uint64_t v) : inner{U384::from_int(v)} {}
    constexpr NewFp256(U384 v) : inner{v} {}

    constexpr explicit operator U384() const
    {
        return inner;
    }

    constexpr NewFp256 operator+(const NewFp256 rhs) const
    {
        return NewFp256(add(inner, rhs.inner));
    }

    constexpr NewFp256 operator-(const NewFp256 rhs) const
    {
        return NewFp256(sub(inner, rhs.inner));
    }

    constexpr NewFp256 operator*(const NewFp256 rhs) const
    {
        return NewFp256(mul(inner, rhs.inner));
    }

    constexpr bool operator==(const NewFp256 rhs) const
    {
        return inner == rhs.inner;
    }

    constexpr bool operator!=(const NewFp256 rhs) const
    {
        return !(inner == rhs.inner);
    }

    constexpr explicit operator uint32_t() const
    {
        return inner.m_limbs[11];
    }

    NewFp256 operator>>(const uint32_t rhs) const
    {
        return NewFp256(inner >> rhs);
    }

    NewFp256 operator<<(const uint32_t rhs) const
    {
        return NewFp256(inner << rhs);
    }

    constexpr static NewFp256 one()
    {
        // TODO find a way to generate on compile time
        const NewFp256 ONE = NewFp256::mul(U384::from_int((uint32_t) 1), R_SQUARED);
        return ONE;
    }

    constexpr NewFp256 to_montgomery() const
    {
        return mul(inner, R_SQUARED);
    }

    // TODO: make method for all fields
    NewFp256 pow(uint32_t exp) const
    {
        // TODO find a way to generate on compile time
        NewFp256 const ONE = one();
        NewFp256 res = ONE;
        NewFp256 power = *this;

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

    NewFp256 inverse() 
    {
        // used addchain
        // https://github.com/mmcloughlin/addchain
        U384 _10 = mul(inner, inner);
        U384 _11 = mul(_10, inner);
        U384 _1100 = sqn<2>(_11);
        U384 _1101 = mul(inner, _1100);
        U384 _1111 = mul(_10, _1101);
        U384 _11001 = mul(_1100, _1101);
        U384 _110010 = mul(_11001, _11001);
        U384 _110011 = mul(inner, _110010);
        U384 _1000010 = mul(_1111, _110011);
        U384 _1001110 = mul(_1100, _1000010);
        U384 _10000001 = mul(_110011, _1001110);
        U384 _11001111 = mul(_1001110, _10000001);
        U384 i14 = mul(_11001111, _11001111);
        U384 i15 = mul(_10000001, i14);
        U384 i16 = mul(i14, i15);
        U384 x10 = mul(_1000010, i16);
        U384 i27 = sqn<10>(x10);
        U384 i28 = mul(i16, i27);
        U384 i38 = sqn<10>(i27);
        U384 i39 = mul(i28, i38);
        U384 i49 = sqn<10>(i38);
        U384 i50 = mul(i39, i49);
        U384 i60 = sqn<10>(i49);
        U384 i61 = mul(i50, i60);
        U384 i72 = mul(sqn<10>(i60), i61);
        U384 x60 = mul(_1000010, i72);
        U384 i76 = sqn<2>(mul(i72, x60));
        U384 x64 = mul(mul(i15, i76), i76);
        U384 i208 = mul(sqn<64>(mul(sqn<63>(mul(i15, x64)), x64)), x64);
        return NewFp256(mul(sqn<60>(i208), x60));
    }

    NewFp256 neg()
    {
        // TODO: can improve
        return NewFp256(sub(U384::from_int((uint32_t)0), inner));
    }

private:
    template<uint32_t N_ACC>

    U384 sqn(U384 base) const {
        U384 result = base;
        #pragma unroll
        for (uint32_t i = 0; i < N_ACC; i++) {
            result = mul(result, result);
        }
        return result;
    }

    // Computes `lhs + rhs mod N`
    // Returns value in range [0,N)
    inline U384 add(const U384 lhs, const U384 rhs) const
    {
        U384 addition = lhs + rhs;
        U384 res = addition;
        // TODO: determine if an if statement here are more optimal

        return res - U384::from_int((uint64_t)(addition >= N)) * N + U384::from_int((uint64_t)(addition < lhs)) * R_SUB_N;
    }

    // Computes `lhs - rhs mod N`
    // Assumes `rhs` value in range [0,N)
    inline U384 sub(const U384 lhs, const U384 rhs) const
    {
        return add(lhs, ((U384)N) - rhs);
    }

    // Computes `lhs * rhs mod M`
    //
    // Essential that inputs are already in the range [0,N) and are in montgomery
    // form. Multiplication performs single round of montgomery reduction.
    //
    // Reference:
    // - https://en.wikipedia.org/wiki/Montgomery_modular_multiplication (REDC)
    // - https://www.youtube.com/watch?v=2UmQDKcelBQ
    constexpr static U384 mul(const U384 a, const U384 b)
    {
        constexpr uint64_t NUM_LIMBS = 12;
        metal::array<uint32_t, NUM_LIMBS> t = {};
        metal::array<uint32_t, 2> t_extra = {};

        U384 q = N;

        for (int i = NUM_LIMBS - 1; i >= 0; i--) {
            // C := 0
            uint32_t c = 0;

            // for j=0 to N-1
            //    (C,t[j]) := t[j] + a[j]*b[i] + C
            for (int j = NUM_LIMBS - 1; j >= 0; j--) {
                uint64_t cs = (uint64_t)t[j] + (uint64_t)a.m_limbs[j] * (uint64_t)b.m_limbs[i] + (uint64_t)c;
                c = cs >> 32;
                t[j] = cs; // low
            }

            // (t[N+1],t[N]) := t[N] + C
            uint64_t cs = (uint64_t)t_extra[1] + (uint64_t)c;
            t_extra[0] = cs >> 32;
            t_extra[1] = cs; // low

            // m := t[0]*q'[0] mod D
            uint32_t m = t[NUM_LIMBS - 1] * MU;

            // (C,_) := t[0] + m*q[0]
            c = ((uint64_t)t[NUM_LIMBS - 1] + (uint64_t)m * (uint64_t)q.m_limbs[NUM_LIMBS - 1]) >> 32;

            // for j=1 to N-1
            //    (C,t[j-1]) := t[j] + m*q[j] + C
            for (int j = NUM_LIMBS - 2; j >= 0; j--) {
                cs = (uint64_t)t[j] + (uint64_t)m * (uint64_t)q.m_limbs[j] + (uint64_t)c;
                c = cs >> 32;
                t[j + 1] = cs; // low
            }

            // (C,t[N-1]) := t[N] + C
            cs = (uint64_t)t_extra[1] + (uint64_t)c;
            c = cs >> 32;
            t[0] = cs; // low

            // t[N] := t[N+1] + C
            t_extra[1] = t_extra[0] + c;
        }

        U384 result {t};

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
