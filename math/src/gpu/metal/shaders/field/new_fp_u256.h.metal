#pragma once

#include "unsigned_int.h.metal"

namespace {
    typedef UnsignedInteger<8> uu256;
}

// taken from the Rust implementation
constexpr static const constant uu256 N = {
    0x8000000,0x11,
    0x0,0x0,
    0x0,0x0,
    0x0,0x1
};
constexpr static const constant uu256 R_SQUARED = {
    0x07FFD4AB,0x5E008810,
    0xFFFFFFFF,0xFF6F8000,
    0x00000001,0x330FFFFF,
    0xFFFFFD73,0x7E000401,
};

// Equates to `(1 << 256) - N`
constexpr static const constant uu256 R_SUB_N = {
    0xf7ffffff, 0xffffffee,
    0xffffffff, 0xffffffff, 
    0xffffffff, 0xffffffff, 
    0xffffffff, 0xffffffff
};

// MU = -N^{-1} mod (2^32)
constexpr static const constant uint64_t MU = 4294967295;

class NewFp256 {
public:
    uu256 inner;
    constexpr NewFp256() = default;
    constexpr NewFp256(uint64_t v) : inner{uu256::from_int(v)} {}
    constexpr NewFp256(uu256 v) : inner{v} {}

    constexpr explicit operator uu256() const
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
        return inner.m_limbs[7];
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
        const NewFp256 ONE = NewFp256::mul(uu256::from_int((uint32_t) 1), R_SQUARED);
        return ONE;
    }

    constexpr NewFp256 to_montgomery()
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
        uu256 _10 = mul(inner, inner);
        uu256 _11 = mul(_10, inner);
        uu256 _1100 = sqn<2>(_11);
        uu256 _1101 = mul(inner, _1100);
        uu256 _1111 = mul(_10, _1101);
        uu256 _11001 = mul(_1100, _1101);
        uu256 _110010 = mul(_11001, _11001);
        uu256 _110011 = mul(inner, _110010);
        uu256 _1000010 = mul(_1111, _110011);
        uu256 _1001110 = mul(_1100, _1000010);
        uu256 _10000001 = mul(_110011, _1001110);
        uu256 _11001111 = mul(_1001110, _10000001);
        uu256 i14 = mul(_11001111, _11001111);
        uu256 i15 = mul(_10000001, i14);
        uu256 i16 = mul(i14, i15);
        uu256 x10 = mul(_1000010, i16);
        uu256 i27 = sqn<10>(x10);
        uu256 i28 = mul(i16, i27);
        uu256 i38 = sqn<10>(i27);
        uu256 i39 = mul(i28, i38);
        uu256 i49 = sqn<10>(i38);
        uu256 i50 = mul(i39, i49);
        uu256 i60 = sqn<10>(i49);
        uu256 i61 = mul(i50, i60);
        uu256 i72 = mul(sqn<10>(i60), i61);
        uu256 x60 = mul(_1000010, i72);
        uu256 i76 = sqn<2>(mul(i72, x60));
        uu256 x64 = mul(mul(i15, i76), i76);
        uu256 i208 = mul(sqn<64>(mul(sqn<63>(mul(i15, x64)), x64)), x64);
        return NewFp256(mul(sqn<60>(i208), x60));
    }

    NewFp256 neg()
    {
        // TODO: can improve
        return NewFp256(sub(uu256::from_int((uint32_t)0), inner));
    }

private:

    template<uint32_t N_ACC>
    uu256 sqn(uu256 base) const {
        uu256 result = base;
#pragma unroll
        for (uint32_t i = 0; i < N_ACC; i++) {
            result = mul(result, result);
        }
        return result;
    }

    // Computes `lhs + rhs mod N`
    // Returns value in range [0,N)
    inline uu256 add(const uu256 lhs, const uu256 rhs) const
    {
        uu256 addition = lhs + rhs;
        uu256 res = addition;
        // TODO: determine if an if statement here are more optimal

        return res - uu256::from_int((uint64_t)(addition >= N)) * N + uu256::from_int((uint64_t)(addition < lhs)) * R_SUB_N;
    }

    // Computes `lhs - rhs mod N`
    // Assumes `rhs` value in range [0,N)
    inline uu256 sub(const uu256 lhs, const uu256 rhs) const
    {
        return add(lhs, ((uu256)N) - rhs);
    }

    // Computes `lhs * rhs mod M`
    //
    // Essential that inputs are already in the range [0,N) and are in montgomery
    // form. Multiplication performs single round of montgomery reduction.
    //
    // Reference:
    // - https://en.wikipedia.org/wiki/Montgomery_modular_multiplication (REDC)
    // - https://www.youtube.com/watch?v=2UmQDKcelBQ
    constexpr static uu256 mul(const uu256 a, const uu256 b)
    {
        constexpr uint64_t NUM_LIMBS = 8;
        metal::array<uint32_t, NUM_LIMBS> t = {};
        metal::array<uint32_t, 2> t_extra = {};

        uu256 q = N;

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
                t[j] = (uint32_t)((cs << 32) >> 32);
            }

            // (t[N+1],t[N]) := t[N] + C
            cs = (uint64_t)t_extra[1] + c;
            t_extra[0] = (uint32_t)(cs >> 32);
            t_extra[1] = (uint32_t)((cs << 32) >> 32);

            // m := t[0]*q'[0] mod D
            uint64_t m = (((uint64_t)t[NUM_LIMBS - 1] * MU) << 32) >> 32;

            // (C,_) := t[0] + m*q[0]
            c = ((uint64_t)t[NUM_LIMBS - 1] + m * (uint64_t)q.m_limbs[NUM_LIMBS - 1]) >> 32;

            // for j=1 to N-1
            //    (C,t[j-1]) := t[j] + m*q[j] + C

            j = NUM_LIMBS - 1;
            while (j > 0) {
                j -= 1;
                cs = (uint64_t)t[j] + m * (uint64_t)q.m_limbs[j] + c;
                c = cs >> 32;
                t[j + 1] = (uint32_t)((cs << 32) >> 32);
            }

            // (C,t[N-1]) := t[N] + C
            cs = (uint64_t)t_extra[1] + c;
            c = cs >> 32;
            t[0] = (uint32_t)((cs << 32) >> 32);

            // t[N] := t[N+1] + C
            t_extra[1] = t_extra[0] + (uint32_t)c;
        }

        uu256 result {t};

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
