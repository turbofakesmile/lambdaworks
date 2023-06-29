#pragma once

namespace uint_tests {
    template<typename U>
    [[kernel]]
    void add(
        constant U& _a [[ buffer(0) ]],
        constant U& _b [[ buffer(1) ]],
        device U& result [[ buffer(2) ]])
    {
        U a = _a;
        U b = _b;

        result = a + b;
    }

    template<typename U>
    [[kernel]]
    void sub(
        constant U& _a [[ buffer(0) ]],
        constant U& _b [[ buffer(1) ]],
        device U& result [[ buffer(2) ]])
    {
        U a = _a;
        U b = _b;

        result = a - b;
    }

    template<typename U>
    [[kernel]]
    void mul(
        constant U& _a [[ buffer(0) ]],
        constant U& _b [[ buffer(1) ]],
        device U& result [[ buffer(2) ]])
    {
        U a = _a;
        U b = _b;

        result = a * b;
    }

    template<typename U>
    [[kernel]]
    void shl(
        constant U& _a [[ buffer(0) ]],
        constant uint64_t& _b [[ buffer(1) ]],
        device U& result [[ buffer(2) ]])
    {
        U a = _a;
        uint64_t b = _b;

        result = a << b;
    }

    template<typename U>
    [[kernel]]
    void shr(
        constant U& _a [[ buffer(0) ]],
        constant uint64_t& _b [[ buffer(1) ]],
        device U& result [[ buffer(2) ]])
    {
        U a = _a;
        uint64_t b = _b;

        result = a >> b;
    }
}
