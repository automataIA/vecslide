---
title: "Singular Value Decomposition"
author: "VecSlide Course"
---

#align(center)[
  #text(size: 48pt)[Singular Value Decomposition]

  #v(1em)

  #text(size: 24pt, weight: "regular")[The Fundamental Matrix Factorization]

  #v(2em)

  #text(size: 18pt, weight: "regular", fill: luma(160))[MIT 18.065 — Matrix Methods in Data Science, Signal Processing, and ML]
]

----

= Why Do We Factorize Matrices?

#v(1em)

- *Data compression* — represent a large matrix with far fewer numbers
- *Noise reduction* — separate meaningful signal from random noise
- *Pattern discovery* — reveal hidden structure in data

#v(2em)

#align(center)[
  #box(inset: 12pt, fill: luma(30), radius: 4pt)[
    #text(size: 18pt)[
      A large matrix \
      $(1000 times 1000)$
    ]
  ]
  #text(size: 28pt, fill: rgb("#4DA6FF"))[ $ arrow.r $ ]
  #box(inset: 12pt, fill: luma(30), radius: 4pt)[
    #text(size: 18pt)[
      Three smaller matrices \
      $U \\ Sigma \\ V^T$
    ]
  ]
  #text(size: 28pt, fill: rgb("#4DA6FF"))[ $ arrow.r $ ]
  #box(inset: 12pt, fill: luma(30), radius: 4pt)[
    #text(size: 18pt)[
      Actionable insight
    ]
  ]
]

----

= The SVD: Every Matrix Has One

#v(1em)

#align(center)[
  #text(size: 40pt)[
    $ A = U Sigma V^T $
  ]
]

#v(1em)

For any $m times n$ matrix $A$:

- $U$ is an $m times m$ *orthogonal* matrix — its columns are orthonormal
- $Sigma$ is an $m times n$ *diagonal* matrix with non-negative entries
- $V$ is an $n times n$ *orthogonal* matrix — its columns are orthonormal

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  Every real or complex matrix admits an SVD — no exceptions.
]

----

= SVD as a Geometric Transformation

#v(1em)

Any linear transformation can be decomposed into three simple operations:

#v(2em)

#align(center)[
  #box(inset: 16pt, fill: luma(40), radius: 8pt)[
    #text(size: 22pt)[Rotate $V^T$]
  ]
  #text(size: 28pt, fill: rgb("#4DA6FF"))[ $ arrow.r.double $ ]
  #box(inset: 16pt, fill: rgb("#4DA6FF").lighten(90%), radius: 8pt)[
    #text(size: 22pt, weight: "bold")[Scale $Sigma$]
  ]
  #text(size: 28pt, fill: rgb("#4DA6FF"))[ $ arrow.r.double $ ]
  #box(inset: 16pt, fill: luma(40), radius: 8pt)[
    #text(size: 22pt)[Rotate $U$]
  ]
]

#v(2em)

The unit circle in $RR^n$ is first rotated by $V^T$, then stretched along coordinate axes by $Sigma$, then rotated into its final position by $U$.

----

= Singular Values: The Diagonal of $Sigma$

#v(1em)

#align(center)[
  $ Sigma = diag(s_1, s_2, ..., s_r, 0, ..., 0) $
]

#v(1em)

Key properties:

- $s_1 >= s_2 >= ... >= s_r > 0$ — the *singular values*, always ordered
- $r = "rank"(A)$ — the number of nonzero singular values equals the rank
- The ratio $s_1 \/ s_r$ is the *condition number* of $A$

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  Large singular values encode the most important structure; small ones encode fine detail or noise.
]

----

= Step 1: Compute $A^T A$

#v(1em)

Starting from $A = U Sigma V^T$:

#align(center)[
  $ A^T A = (U Sigma V^T)^T (U Sigma V^T) = V Sigma^T U^T U Sigma V^T = V (Sigma^T Sigma) V^T $
]

#v(1em)

Since $U$ is orthogonal: $U^T U = I$.

$Sigma^T Sigma$ is a diagonal matrix with entries $s_i^2$ on the diagonal.

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 10pt, radius: 4pt)[
  *Key insight:* The eigenvectors of $A^T A$ give us $V$, and the eigenvalues are $s_i^2$.
]

----

= Step 2: Compute $U$ from $V$

#v(1em)

We *could* find $U$ from $A A^T$, but this risks *sign ambiguity* (eigenvectors are only unique up to sign). Instead, we use the fact that $A V = U Sigma$:

#align(center)[
  $ A v_i = s_i u_i quad arrow.r.double quad u_i = 1/s_i A v_i $
]

#v(1em)

Since we already know the singular values $s_i$ and the right singular vectors $v_i$, we directly compute the left singular vectors $u_i$.

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 10pt, radius: 4pt)[
  *Key insight:* This "Strang approach" mathematically locks the correct geometric correlation between $U$ and $V$, avoiding any sign mismatch.
]

----

= Step 3: Singular Values from Eigenvalues

#v(2em)

#align(center)[
  #text(size: 36pt)[
    $ s_i = sqrt(lambda_i) $
  ]
]

#v(2em)

where $lambda_i$ are the eigenvalues of $A^T A$ (or equivalently of $A A^T$).

#v(1em)

The singular values are simply the square roots of the eigenvalues — this is the bridge between the SVD and the eigendecomposition.

----

= The Full Picture: $A = U Sigma V^T$

#v(1em)

#align(center)[
  #table(
    columns: (auto, auto, auto, auto, auto, auto, auto),
    stroke: none,
    align: center,
    inset: 10pt,

    // Row 1: dimensions
    [*A*], [], [*U*], [], [*$Sigma$*], [], [*$V^T$*],
    [$(m times n)$], [], [$(m times m)$], [], [$(m times n)$], [], [$(n times n)$],
  )
]

#v(1em)

#align(center)[
  #table(
    columns: 5,
    stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("#4DA6FF")) },
    inset: 8pt,
    align: center,

    // Header
    [Column space], [], [Scaling], [], [Row space],
    // Content
    [$u_1, ..., u_m$], $arrow.r$, [$s_1 >= ... >= s_r$], $arrow.r$, [$v_1, ..., v_n$],
  )
]

#text(size: 18pt, fill: luma(160))[
  The columns of $U$ span the column space of $A$; the columns of $V$ span the row space.
]

----

= Low-Rank Approximation

#v(1em)

Keep only the top $k$ singular values:

#align(center)[
  $ A_k = sum_(i=1)^k s_i \. u_i \. v_i^T $
]

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 10pt, radius: 4pt)[
  *Eckart-Young-Mirsky Theorem:* $A_k$ is the best rank-$k$ approximation of $A$ under both the Frobenius norm and the spectral norm.
]

#v(1em)

#align(center)[
  #table(
    columns: (1fr, 1fr, 1fr),
    stroke: none,
    inset: 8pt,
    align: center,
    [*k = 1*], [*k = 5*], [*k = 20*],
    [Very blurry], [Recognizable], [Nearly perfect],
  )
]

#text(size: 18pt, fill: luma(160))[
  Compression ratio: storing $A_k$ requires only $k(m + n + 1)$ values instead of $m times n$.
]

----

= The Pseudoinverse via SVD

#v(1em)

The Moore-Penrose pseudoinverse is defined as:

#align(center)[
  $ A^+ = V Sigma^+ U^T $
]

where $Sigma^+$ inverts the nonzero singular values:

#align(center)[
  $ Sigma^+ = diag(1\/s_1, 1\/s_2, ..., 1\/s_r, 0, ..., 0) $
]

#v(1em)

This gives the least-squares solution even when $A$ is not square or not full rank:

#align(center)[
  $ x = A^+ b $
]

#v(1em)

#text(size: 18pt, fill: rgb("#4DA6FF"))[
  Applications: linear regression, system identification, signal reconstruction.
]

----

= Key Takeaways: SVD

#v(1em)

+ Every matrix $A$ decomposes as $A = U Sigma V^T$ — no exceptions
+ $U$ and $V$ are orthogonal; $Sigma$ is diagonal with $s_1 >= s_2 >= ... >= s_r > 0$
+ Singular values come from $sqrt("eigenvalues of" \. A^T A)$
+ Low-rank approximation: keep top $k$ singular values — optimal by Eckart-Young
+ Used everywhere: PCA, image compression, least squares, recommender systems

#v(1em)

#text(size: 18pt, fill: luma(160))[
  The SVD is the Swiss Army knife of linear algebra — one factorization, countless applications.
]

----

#align(center)[
  #v(3em)

  #text(size: 36pt)[Next: Backpropagation]

  #v(1em)

  #text(size: 20pt, weight: "regular")[
    We've seen how matrices reveal structure in data. \
    Next, we'll see how chains of matrix operations \
    power neural networks — and how to compute their \
    gradients efficiently.
  ]
]
