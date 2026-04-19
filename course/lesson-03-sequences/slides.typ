---
title: "Convergence of Sequences"
author: "VecSlide Course"
---

#align(center)[
  #text(size: 48pt)[Convergence of Sequences]

  #v(1em)

  #text(size: 24pt, weight: "regular")[The Foundation of Analysis]

  #v(2em)

  #text(size: 18pt, weight: "regular", fill: luma(160))[MIT 18.100B — Real Analysis]
]

----

= Why Do We Study Sequences?

#v(1em)

Sequences appear everywhere in science and engineering:

#v(1em)

- *Machine learning:* training losses form a sequence $L_1, L_2, L_3, ...$ — does it converge to a minimum?
- *Numerical methods:* iterative algorithms produce sequences of approximations — do they approach the true answer?
- *Foundations:* limits of sequences are the building block for series, continuity, derivatives, and integrals

#v(1em)

#align(center)[
  #box(inset: 14pt, fill: rgb("#4DA6FF").lighten(90%), radius: 6pt)[
    #text(size: 20pt)[
      The central question: does a given sequence "settle down" to a limiting value?
    ]
  ]
]

----

= What Is a Sequence?

#v(1em)

#align(center)[
  A *sequence* is a function $a: NN -> RR$, written as:

  $ (a_n)_(n=1)^infinity = a_1, \. a_2, \. a_3, \. ... $
]

#v(1em)

An infinite ordered list of real numbers.

#v(1em)

Examples:

#align(center)[
  #table(
    columns: (auto, auto, auto),
    stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("#4DA6FF")) },
    inset: 8pt,
    align: left,
    [*Sequence*], [*First terms*], [*Behavior*],
    [$a_n = 1\/n$], [$1, 1\/2, 1\/3, ...$], [Approaches 0],
    [$a_n = (-1)^n$], [$-1, 1, -1, 1, ...$], [Oscillates],
    [$a_n = n^2$], [$1, 4, 9, 16, ...$], [Diverges to infinity],
  )
]

----

= Formal Definition of Convergence

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 12pt, radius: 6pt)[
  A sequence $(a_n)$ *converges* to $L in RR$ if:

  #align(center)[
    $ forall epsilon > 0, \. exists N in NN \: "such that" \: forall n >= N : |a_n - L| < epsilon $
  ]
]

#v(1em)

In words: for *any* distance $epsilon$, no matter how small, the terms of the sequence eventually stay within $epsilon$ of $L$.

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  We write $(a_n) -> L$ or $lim_(n -> infinity) a_n = L$.
]

----

= Visualizing Convergence: The $epsilon$-Tube

#v(1em)

The definition has a clear geometric meaning:

#v(2em)

#align(center)[
  #table(
    columns: (1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr, 1fr),
    stroke: none,
    inset: 4pt,
    align: center,
    // Axis labels
    [$n$:], [1], [2], [3], [4], [5], [6], [7], [8], [...],
    // Values approaching L
    [$a_n$:], [$bullet$], [$bullet$], [$bullet$], [$bullet$], [$bullet$], [$bullet$], [$bullet$], [$bullet$], [],
  )
]

#v(1em)

#align(center)[
  #box(inset: 10pt, fill: luma(30), radius: 4pt, width: 80%)[
    #text(size: 16pt)[
      Dashed lines at $L + epsilon$ and $L - epsilon$ form a tube. \
      After index $N$, all points stay inside the tube. \
      The value $N$ depends on $epsilon$ — smaller $epsilon$ means larger $N$.
    ]
  ]
]

----

= Example: $a_n = 1\/n$

#v(1em)

Claim: $1\/n -> 0$.

#v(1em)

*Proof.* Given $epsilon > 0$, choose $N > 1\/epsilon$.

Then for all $n >= N$:

#align(center)[
  #text(size: 28pt, fill: rgb("#4DA6FF"))[
    $ |1\/n - 0| = 1\/n <= 1\/N < epsilon \. "square" $
  ]
]

#v(1em)

#v(1em)

#text(size: 18pt, fill: luma(160))[
  The key step: finding $N$ in terms of $epsilon$. For $epsilon = 0.01$, we need $N > 100$. For $epsilon = 0.001$, we need $N > 1000$. The pattern is clear.
]

----

= Limits Are Unique

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 12pt, radius: 6pt)[
  *Theorem.* If $(a_n) -> L_1$ and $(a_n) -> L_2$, then $L_1 = L_2$.
]

#v(1em)

*Proof sketch.* Suppose $L_1 != L_2$. Choose $epsilon = |L_1 - L_2| \/ 2$.

By the definition, for large enough $n$, $a_n$ must be within $epsilon$ of *both* $L_1$ and $L_2$.

But the distance between $L_1$ and $L_2$ is $2 epsilon$, and by the triangle inequality, $a_n$ cannot be within $epsilon$ of both simultaneously.

#v(1em)

#text(size: 18pt, fill: rgb("#4DA6FF"))[
  A sequence cannot have two different limits — the limit is well-defined.
]

----

= The Problem: What If We Don't Know the Limit?

#v(2em)

#align(center)[
  #text(size: 24pt, weight: "regular")[
    The definition of convergence requires knowing $L$ in advance.
  ]
]

#v(2em)

#align(center)[
  #text(size: 24pt, weight: "regular")[
    But in practice, we often want to prove a sequence converges \
    *without knowing what it converges to*.
  ]
]

#v(2em)

#align(center)[
  #box(inset: 14pt, fill: rgb("#4DA6FF").lighten(90%), radius: 6pt)[
    #text(size: 20pt)[
      Can we guarantee convergence from the *behavior of the terms themselves*?
    ]
  ]
]

----

= Cauchy Sequences: Convergence Without a Limit

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 12pt, radius: 6pt)[
  A sequence $(a_n)$ is *Cauchy* if:

  #align(center)[
    $ forall epsilon > 0, \. exists N in NN : forall m, n >= N, \: |a_m - a_n| < epsilon $
  ]
]

#v(1em)

#align(center)[
  #box(inset: 10pt, fill: luma(30), radius: 4pt, width: 90%)[
    #text(size: 16pt)[
      *Visualizing Cauchy:* Unlike the $epsilon$-tube fixed to $L$, picture a freely moving "cluster" of width $epsilon$. As $n$ grows, the sequence terms pack into tighter and tighter clusters, squeezing together without needing an anchor point. 
    ]
  ]
]

#align(center)[
  #table(
    columns: (1fr, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("#4DA6FF")) },
    inset: 8pt,
    align: center,
    [*Convergence*], [*Cauchy*],
    [$|a_n - L| < epsilon$], [$|a_m - a_n| < epsilon$],
    [Distance to a *known* target], [Distance *between* each other],
  )
]

----

= Completeness of $RR$: Cauchy $<=>$ Convergent

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 12pt, radius: 6pt)[
  *Theorem (Completeness of $RR$).* \
  In $RR$, a sequence converges if and only if it is Cauchy.
]

#v(1em)

This is the *completeness axiom* of the real numbers.

#v(1em)

It *fails* in $QQ$ (the rationals):

#align(center)[
  #table(
    columns: (auto, auto, auto),
    stroke: none,
    inset: 8pt,
    align: left,
    [*Sequence*], [*$in QQ$?*], [*Limit $in QQ$?*],
    [$a_n = (1 + 1\/n)^n$], [Yes], [No — it approaches $e approx 2.718...$],
    [$a_1 = 1, a_(n+1) = (a_n + 2\/a_n)\/2$], [Yes], [No — it approaches $sqrt(2)$],
  )
]

#v(1em)

#text(size: 18pt, fill: rgb("#4DA6FF"))[
  The reals have "no gaps" — every Cauchy sequence has a home. The rationals have gaps at every irrational number.
]

----

= Application: Fixed-Point Iteration

#v(1em)

Consider the iterative scheme $x_(n+1) = g(x_n)$ for solving $g(x) = x$.

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 10pt, radius: 4pt)[
  *Banach Fixed-Point Theorem.* If $g$ is a *contraction*:

  #align(center)[
    $ |g(x) - g(y)| <= c \. |x - y| quad "for some" \: c < 1 "and all" \: x, y $
  ]

  then $(x_n)$ is Cauchy and converges to the unique fixed point $x_star$.
]

#v(1em)

#align(center)[
  $ x_star = lim_(n -> infinity) x_n, quad g(x_star) = x_star $
]

#text(size: 18pt, fill: luma(160))[
  The rate of convergence is geometric: $|x_n - x_star| <= c^n \/ (1 - c) \. |x_1 - x_0|$.
]

----

= Connection to Machine Learning

#v(1em)

In gradient descent, the sequence of losses $L_0, L_1, L_2, ...$ is generated iteratively:

#align(center)[
  $ W_(t+1) = W_t - eta \. nabla L(W_t) quad arrow.r.double quad L_(t+1) <= L_t $
]

#v(1em)

Under appropriate conditions:

#align(center)[
  #table(
    columns: (auto, auto),
    stroke: none,
    inset: 6pt,
    align: left,
    [1.], [$L$ is *convex* and differentiable],
    [2.], [Learning rate $eta$ is small enough],
    [3.], [Gradients are Lipschitz continuous],
  )
]

#v(1em)

Then $(L_t)$ converges to the global minimum — guaranteed by the same Cauchy/completeness theory.

#v(1em)

#text(size: 18pt, fill: rgb("#4DA6FF"))[
  Backpropagation computes the gradients; convergence theory guarantees the iteration reaches the optimum.
]

----

= Key Takeaways: Convergence

#v(1em)

+ A sequence $(a_n)$ converges to $L$ if its terms eventually stay within any $epsilon$-neighborhood of $L$
+ The Cauchy criterion replaces knowledge of $L$ with pairwise closeness of terms
+ In $RR$, Cauchy and convergent are equivalent — this is the completeness axiom
+ Fixed-point iteration converges when the mapping is a contraction ($c < 1$)
+ These ideas underpin iterative methods, numerical algorithms, and ML training

#v(1em)

#text(size: 18pt, fill: luma(160))[
  Convergence theory is what guarantees that optimization "gets there" — not just that it moves in the right direction.
]

----

#align(center)[
  #v(2em)

  #text(size: 36pt)[End of Module]

  #v(1em)

  #text(size: 20pt, weight: "regular")[
    In these three lessons, we moved from *linear algebra* \
    (SVD) through *deep learning* (backpropagation) to \
    *real analysis* (convergence).
  ]

  #v(1em)

  #text(size: 20pt, weight: "regular")[
    Together, they form the mathematical backbone \
    of modern machine learning.
  ]
]
