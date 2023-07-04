#pragma once

#include "../fields/unsigned_int.h.metal"

namespace {
    typedef UnsignedInteger<8> U256;
}

[[kernel]]
void test_uint_add(
    constant U256& _a [[ buffer(0) ]],
    constant U256& _b [[ buffer(1) ]],
    device U256& result [[ buffer(2) ]])
{
    U256 a = _a;
    U256 b = _b;

    result = a + b;
}

[[kernel]]
void test_uint_sub(
    constant U256& _a [[ buffer(0) ]],
    constant U256& _b [[ buffer(1) ]],
    device U256& result [[ buffer(2) ]])
{
    U256 a = _a;
    U256 b = _b;

    result = a - b;
}

[[kernel]]
void test_uint_prod(
    constant U256& _a [[ buffer(0) ]],
    constant U256& _b [[ buffer(1) ]],
    device U256& result [[ buffer(2) ]])
{
    U256 a = _a;
    U256 b = _b;

    result = a * b;
}

[[kernel]]
void test_uint_shl(
    constant U256& _a [[ buffer(0) ]],
    constant uint64_t& _b [[ buffer(1) ]],
    device U256& result [[ buffer(2) ]])
{
    U256 a = _a;
    uint64_t b = _b;

    result = a << b;
}

[[kernel]]
void test_uint_shr(
    constant U256& _a [[ buffer(0) ]],
    constant uint64_t& _b [[ buffer(1) ]],
    device U256& result [[ buffer(2) ]])
{
    U256 a = _a;
    uint64_t b = _b;

    result = a >> b;
}
