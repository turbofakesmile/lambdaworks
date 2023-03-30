# Frequently Asked Questions

## Why use roots of unity?

Whenever we interpolate or evaluate trace, boundary and constraint polynomials, we use some $2^n$-th roots of unity. There are a few reasons for this:

- Using roots of unity means we can use the [Fast Fourier Transform](https://en.wikipedia.org/wiki/Fast_Fourier_transform) and its inverse to evaluate and interpolate polynomials. This method is much faster than the naive Lagrange interpolation one. Since a huge part of the STARK protocol involves both evaluating and interpolating, this is a huge performance improvement.
- When computing boundary and constraint polynomials, we divide them by their `zerofiers`, polynomials that vanish on a few points (the trace elements where the constraints do not hold). These polynomials take the form

    $$
    Z(X) = \prod (X - x_i)
    $$

    where the $x_i$ are the points where we want it to vanish.

    When implementing this, evaluating this polynomial can be very expensive as it involves a huge product. However, if we are using roots of unity, we can use the following trick. The vanishing polynomial for all the $2^n$ roots of unity is

    $$
    X^{2^n} - 1
    $$

    Instead of expressing the zerofier as a product of the places where it should vanish, we express it as the vanishing polynomial above divided by the `exemptions` polynomial; the polynomial whose roots are the places where constraints don't need to hold. 

    $$
    Z(X) = \dfrac{X^{2^n} - 1}{\prod{(X - e_i)}}
    $$

    where the $e_i$ are now the points where we don't want it to vanish. This `exemptions` polynomial in the denominator is usually much smaller, and because the vanishing polynomial in the numerator is only two terms, evaluating it is really fast.

## What is a primitive root of unity?

The $n$-th roots of unity are the numbers $x$ that satisfy

$$
x^n = 1
$$

There are $n$ such numbers, because they are the roots of the polynomial $X^n - 1$. The set of $n$-th roots of unity always has a `generator`, a root $g$ that can be used to obtain every other root of unity by exponentiating. What this means is that the set of $n$-th roots of unity is

$$
\{g^i : 0 \leq i < n\}
$$

Any such generator `g` is called a *primitive root of unity*. It's called primitive because it allows us to recover any other root.

Here are a few important things to keep in mind, some of which we use throughout our implementation:

- There are always several primitive roots. If $g$ is primitive, then any power $g^k$ with $k$ coprime with $n$ is also primitive. As an example, if $g$ is a primitive $8$-th root of unity, then $g^3$ is also primitive.
- We generally will not care about which primitive root we choose; what we do care about is being *consistent*. We should always choose the same one throughout our code, otherwise computations will go wrong.
- Because $g^n = 1$, the powers of $g$ wrap around. This means

    $$
    g^{n + 1} = g \\
    g^{n + 2} = g^2
    $$

  and so on.
- If $w$ is a primitive $2^{n + 1}$-th root of unity, then $w^2$ is a primitive $2^n$-th root of unity. In general, if $w$ is a primitive $2^{n + k}$-th primitive root of unity, then $w^{2^k}$ is a primitive $2^n$-th root of unity.

## Why use Cosets?

When we perform `FRI` on the `DEEP` composition polynomial, the low degree extension we use is not actually over a set of higher roots of unity than the ones used for the trace, but rather a *coset* of it. A coset is simply a set of numbers all multiplied by the same element. We call said element the `offset`. In our case, a coset of the $2^n$-th roots of unity with primitive root $\omega$ and offset `h` is the set

$$
\{h \omega^i : 0 \leq i < 2^n\}
$$

So why not just do the LDE without the offset? The problem is in how we construct and evaluate the composition polynomial `H`. Let's say our trace polynomial was interpolated over the $2^n$-th roots of unity with primitive root $g$, and we are doing the LDE over the $2^{n + 1}$-th roots of unity with primitive root $\omega$, so $\omega^2 = g$ (i.e. the blowup factor is `2`).

Recall that `H` is a sum of terms that include boundary and transition constraint polynomials, and each one of them includes a division by a `zerofier`; a polynomial that vanishes on some roots of unity $g^i$. This is because the zerofier is what tells us which rows of the trace our constraint should apply on.

When doing `FRI`, we have to provide evaluations over the LDE domain we are using. If we don't include the offset, our domain is

$$
\{\omega^i : 0 \leq i < 2^{n + 1}\}
$$

Note that, because $w^2 = g$, some of the elements on this set (actually, half of them) are powers of $g$. If while doing `FRI` we evalaute `H` on them, the zerofier could vanish and we'd be dividing by zero. We introduce the offset to make sure this can't happen.

NOTE: a careful reader might note that we can actually evaluate `H` on the elements $g^i$, since on a valid trace the zerofiers will actually divide the polynomials on their numerator. The problem still remains, however, because of performance. We don't want to do polynomial division if we don't need to, it's much cheaper to just evaluate numerator and denominator and then divide. Of course, this only works if the denominator doesn't vanish; hence, cosets.

----------

## Polynomials

A univariate polynomial is a mathematical object of the form
$$p(x) = a_0+a_1 x + a_2 x^2 + \cdots +a_n x^n$$
where the $a_k$ are the coefficients of the polynomial (you can think of them as numbers, such as real, complex or integers). The greatest power of $x$ such that $a_i$ is non-zero is the degree of the polynomial. For example,
$$ p(x) = 1 + x + 2x^2 + x^3$$
is a polynomial of degree $3$ and
$$ p(x) = 5 $$ has degree zero.

Polynomials can be added or multiplied, equipping them with a ring structure. Addition is done coefficient-wise, while multipliciation involves the application of the distributive property and them summing coefficient-wise. For example,
$p(x) = 3 + 5x + 2x^2$
$q(x) = 1 + 2x^2$
then
$s(x) = p(x) + q(x) = 3 + 5x + 2x^2 + 1 + 2x^2$
$s(x) = (3+1) + 5x + (2 + 2)x^2 = 4 + 5x + 4x^2$
and
$m(x) = p(x)q(x) = (3 + 5x + 2x^2 )(1 + 2x^2)$
$m(x) = 3 + 6 x^2 + 5x + 10x^3 + 2x^2 + 4x^4$
$m(x) = 3 + 5x^2 + 8x^2 + 10x^3 + 4x^4$
The operations between coefficients follow the addition and multiplication rules.

Polynomials can be represented in two common ways:
1. In coefficient form, by giving a vector of its coefficients, $(a_0, a_1 , a_2 , ... , a_n )$.
2. In evaluation form, given by the evaluations of the polynomial over $n+1$ distinct points $(p(x_0), p(x_1) , ..., p(x_n) )$.

We can transform from coefficients to evaluations by choosing $n+1$ points and evaluating, or by taking the $n+1$ evaluations and interpolating. This is possible due to the [interpolation theorem](https://en.wikipedia.org/wiki/Polynomial_interpolation#Interpolation_theorem). Interpolation can be done very efficiently using the Fast-Fourier Transform.

## Polynomial division

In an analogous way to integers, we can define what it means that a polynomial $p(x)$ is divisible by $d(x)$. We say $d(x)$ divides $p(x)$ if there exists a polynomial $q(x)$ such that
$$ p(x) = q(x) d(x)$$
If the polynomial is not divisible, then there is another polynomial $r(x)$ with degree less than the degree of $d(x)$ such that
$$ p(x) = q(x) d(x) + r(x)$$.

If we want to make the division in this case, the result is not a polynomial, but a rational function.

## Roots of a polynomial

We say that $x_0$ is a root of the polynomial if $p( x_0 ) = 0$. The polynomial can be factored as $p (x) = (x - x_0)Q(x)$, where $Q(x)$ has lower degree than $p(x)$. This implies that the polynomial $x-x_0$ divides $p(x)$.

A consequence of this is that if $p (a) = b$, then $p(x) - b$ is divisible by $x - a$. This is one of the key results to build STARKs. Just think of the new polynomial, $w(x) = p(x) - b$, which has a root on $x = a$, since 
$$w(a) = p(a) - b = b - b = 0$$

## Low-degree extension

We know a polynomial of degree $n$ is specified by giving $n+1$ evaluations of the polynomial. If we give more than $n+1$ evaluations, we recover exactly the same polynomial, just because the information is redundant. If someone changed slightly one of the evaluations, the interpolating polynomial is radically different, since two polynomials can agree at most on $m$ points, where $m$ is the highest degree of the polynomial. This is useful because we can find easily when there is some error or change in the evaluations of the polynomial (for example, this works in error correcting codes) or if we have a dishonest prover in the case of STARKs.

## Merkle trees

## Fast-Fourier Transform

TODO:
- What's the ce blowup factor?
- What's the out of domain frame?
