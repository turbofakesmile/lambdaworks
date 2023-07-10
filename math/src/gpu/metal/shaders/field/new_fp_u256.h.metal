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
    0x07ffd4ab,0x5e008810,
    0xffffffff,0xff6f8000,
    0x00000001,0x330fffff,
    0xfffffd73,0x7e000401
};

// Equates to `(1 << 256) - N`
constexpr static const constant U256 R_SUB_N = {
    0xf7ffffff,0xffffffee,
    0xffffffff,0xffffffff,
    0xffffffff,0xffffffff,
    0xffffffff,0xffffffff
};

// MU = -N^{-1} mod (2^32)
constexpr static const constant uint64_t MU = 4294967295;

class NewFp256 {
public:
    U256 inner;
    constexpr NewFp256() = default;
    constexpr NewFp256(uint64_t v) : inner{U256::from_int(v)} {}
    constexpr NewFp256(U256 v) : inner{v} {}

    constexpr explicit operator U256() const
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
        const NewFp256 ONE = NewFp256::mul(U256::from_int((uint32_t) 1), R_SQUARED);
        return ONE;
    }

    constexpr NewFp256 to_montgomery() const
    {
        return mul(inner, R_SQUARED);
    }

    NewFp256 inverse() 
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
        return NewFp256(mul(sqn<60>(i208), x60));
    }

    NewFp256 neg()
    {
        // TODO: can improve
        return NewFp256(sub(U256::from_int((uint32_t)0), inner));
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

    constexpr static U256 mul(const U256 a, const U256 b)
    {
        constexpr uint64_t NUM_LIMBS = 8;
        metal::array<uint32_t, NUM_LIMBS> t = {};
        metal::array<uint32_t, 2> t_extra = {};

        U256 q = N;

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

        U256 result {t};

        // TODO: assuming the integer represented by
        // [t_extra[1], t[0], ..., t[NUM_LIMBS - 1]] is at most
        // 2q in any case.
        uint64_t mod = (t_extra[0] > 0) || (q <= result);
        return result - (q * U256::from_int(mod));
        // an if statement was replaced for the previous expression because,
        // for some strange reason, it was provoking an incorrect result.
    }
};
