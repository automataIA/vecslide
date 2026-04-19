---
title: "Backpropagation and Computational Graphs"
author: "VecSlide Course"
---

#align(center)[
  #text(size: 48pt)[Backpropagation and Computational Graphs]

  #v(1em)

  #text(size: 24pt, weight: "regular")[How Neural Networks Learn]

  #v(2em)

  #text(size: 18pt, weight: "regular", fill: luma(160))[Stanford CS231n — Deep Learning for Computer Vision]
]

----

= How Does a Neural Network Learn?

#v(1em)

The learning loop:

#v(1em)

#align(center)[
  #box(inset: 14pt, fill: rgb("#4DA6FF").lighten(90%), radius: 6pt)[
    *1. Forward pass*
  ]
  #h(0.5em)
  $ arrow.r $
  #h(0.5em)
  #box(inset: 14pt, fill: luma(40), radius: 6pt)[
    *2. Compute loss*
  ]
  #h(0.5em)
  $ arrow.r $
  #h(0.5em)
  #box(inset: 14pt, fill: rgb("#4DA6FF").lighten(90%), radius: 6pt)[
    *3. Backward pass*
  ]
  #h(0.5em)
  $ arrow.r $
  #h(0.5em)
  #box(inset: 14pt, fill: luma(40), radius: 6pt)[
    *4. Update weights*
  ]
]

#v(2em)

A network with millions of parameters needs an efficient way to compute the gradient of the loss with respect to *every single weight*.

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  Backpropagation does exactly this — using the chain rule.
]

----

= The Chain Rule: The Engine of Backpropagation

#v(1em)

From calculus, the chain rule for composed functions:

#align(center)[
  $ (f @ g)'(x) = f'(g(x)) \. g'(x) $
]

#v(1em)

In Leibniz notation, if $z$ depends on $y$ and $y$ depends on $x$:

#align(center)[
  #text(size: 28pt, fill: rgb("#4DA6FF"))[
    $ (partial z) / (partial x) = (partial z) / (partial y) \. (partial y) / (partial x) $
  ]
]

#v(1em)

The total derivative is the *product* of local derivatives along the path.

#text(size: 18pt, fill: luma(160))[
  Backpropagation applies this formula systematically to every node in a computational graph.
]

----

= A Simple Computational Graph

#v(1em)

Consider computing $f = x y + z$ with $x = 2$, $y = 3$, $z = 5$:

#v(2em)

#align(center)[
  #table(
    columns: (1fr, 1fr, 1fr),
    stroke: none,
    inset: 10pt,
    align: center,

    // Row 1: inputs
    [$x = 2$], [$y = 3$], [$z = 5$],
    // Arrows
    [$arrow.r$], [$arrow.r$], [],
    // Operation
    [$times$], [], [],
    // Row: intermediate
    [$q = x y$], [], [],
    // Arrow
    [$arrow.r$], [], [$arrow.r$],
    // Operation
    [$+$], [], [],
    // Output
    [$f = q + z$], [], [],
  )
]

#v(1em)

Each node is a simple operation. The graph makes the flow of data explicit.

----

= Forward Pass: Computing the Output

#v(1em)

Step by step through the graph:

#v(1em)

#align(center)[
  #table(
    columns: (auto, auto, auto, auto),
    stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("#4DA6FF")) },
    inset: 8pt,
    align: center,
    [*Step*], [*Expression*], [*Value*], [*Node*],
    [1], [$q = x times y$], [$6$], [multiply],
    [2], [$f = q + z$], [#text(fill: rgb("#4DA6FF"))[*$11$*]], [add],
  )
]

#v(1em)

The forward pass computes outputs from inputs, flowing left to right through the graph.

#text(size: 18pt, fill: luma(160))[
  This is exactly what happens when a neural network processes data — each layer computes its output and passes it forward.
]

----

= Backward Pass Starts at the Output

#v(1em)

The backward pass begins with a seed gradient:

#align(center)[
  #text(size: 36pt)[
    $ (partial f)/(partial f) = 1 $
  ]
]

#v(1em)

This is the starting point. From the output, gradients flow *backward* through the graph, accumulating via the chain rule.

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  At each node, we compute: upstream gradient $times$ local gradient.
]

----

= Gradient Through Addition: $f = q + z$

#v(1em)

The addition node $f = q + z$ has simple local gradients:

#align(center)[
  $ (partial f)/(partial q) = 1, quad (partial f)/(partial z) = 1 $
]

#v(1em)

Addition is a *gradient distributor* — it passes the upstream gradient through unchanged to both inputs.

#v(1em)

#align(center)[
  #table(
    columns: (auto, auto, auto, auto),
    stroke: none,
    inset: 6pt,
    align: center,
    [Upstream], [$times$], [Local], [= Result],
    [$1.0$], [$times$], [$1$], [#text(fill: rgb("#4DA6FF"))[$1.0 arrow.r q$]],
    [$1.0$], [$times$], [$1$], [#text(fill: rgb("#4DA6FF"))[$1.0 arrow.r z$]],
  )
]

----

= Gradient Through Multiplication: $q = x y$

#v(1em)

The multiplication node has a *swap-and-scale* pattern:

#align(center)[
  $ (partial q)/(partial x) = y = 3, quad (partial q)/(partial y) = x = 2 $
]

#v(1em)

Applying the chain rule with the upstream gradient from the addition node:

#align(center)[
  #text(size: 28pt, fill: rgb("#4DA6FF"))[
    $ (partial f)/(partial x) = (partial f)/(partial q) \. (partial q)/(partial x) = 1 \. 3 = 3 $
  ]
]

#v(1em)

#align(center)[
  #text(size: 28pt, fill: rgb("#4DA6FF"))[
    $ (partial f)/(partial y) = (partial f)/(partial q) \. (partial q)/(partial y) = 1 \. 2 = 2 $
  ]
]

#text(size: 18pt, fill: luma(160))[
  Multiplication swaps: the gradient with respect to x is scaled by y, and vice versa.
]

----

= The General Pattern: Local Gradients

#v(1em)

Each operation has a known local gradient:

#align(center)[
  #table(
    columns: (auto, auto, auto),
    stroke: (x, y) => if y == 0 { (bottom: 1pt + rgb("#4DA6FF")) },
    inset: 10pt,
    align: left,
    [*Operation $f$*], [*$(partial f)/(partial x)$*], [*$(partial f)/(partial y)$*],
    [$x + y$], [$1$], [$1$],
    [$x \. y$], [$y$], [$x$],
    [$sigma(x)$], [$sigma(x)(1 - sigma(x))$], [---],
    [$"max"(x, y)$], [$cases(1 "if" x>y, 0 "else")$], [$cases(1 "if" y>x, 0 "else")$],
  )
]

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  Each node only needs its *local* gradient — the graph structure handles the rest.
]

----

= A Neuron as a Computational Graph

#v(1em)

A single artificial neuron is a chain of operations:

#v(1em)

#align(center)[
  #table(
    columns: (1fr, 1fr, 1fr, 1fr),
    stroke: none,
    inset: 8pt,
    align: center,
    // Row 1: labels
    [*Inputs*], [*Linear*], [*Activation*], [*Output*],
    // Row 2: formulas
    [$x_1, x_2, ..., x_n$], [$z = w^T x + b$], [$a = sigma(z)$], [$hat(y) = a$],
  )
]

#v(1em)

The weights $w$ and bias $b$ are the *learnable parameters*. Backpropagation computes:

#align(center)[
  $ (partial "Loss")/(partial w_i) quad "and" quad (partial "Loss")/(partial b) $
]

#text(size: 18pt, fill: luma(160))[
  A full neural network is just thousands of these neurons connected in layers — the same backprop algorithm scales.
]

----

= Backpropagation Through One Layer

#v(1em)

For a layer $z = W x + b$, we need the gradients $(partial L)/(partial W)$ and $(partial L)/(partial x)$. Writing full Jacobian matrices is messy. Instead, we use *Dimension Balancing (Shape Matching)*:

#v(1em)

#box(fill: rgb("#4DA6FF").lighten(90%), inset: 10pt, radius: 4pt)[
  *The Golden Rule:* The gradient of the loss with respect to any variable must have the *exact same shape* as that variable.
]

#v(1em)

Since the upstream gradient is $(partial L)/(partial z)$:

#align(center)[
  #table(
    columns: (auto, auto, auto),
    stroke: none,
    inset: 6pt,
    align: left,
    [1.], [Gradient w.r.t $W$:], [$(partial L)/(partial W) = (partial L)/(partial z) \. x^T $],
    [2.], [Gradient w.r.t $x$:], [$(partial L)/(partial x) = W^T \. (partial L)/(partial z) $],
  )
]

#v(1em)

#text(size: 18pt, fill: luma(160))[
  We just arrange the terms and transposes so the resulting matrix dimensions match $W$ and $x$ perfectly. No tensor calculus needed!
]

----

= Deep Networks: Chaining Layers

#v(1em)

A deep network stacks $L$ layers:

#align(center)[
  #table(
    columns: (1fr, 1fr),
    stroke: none,
    inset: 6pt,
    align: left,
    [*Forward (top $arrow.t$ bottom)*], [*Backward (bottom $arrow.b$ top)*],
    [$z_1 = W_1 x + b_1$], [$(partial L)/(partial z_1)$ via chain rule],
    [$a_1 = sigma(z_1)$], [$(partial L)/(partial a_1) arrow.r (partial L)/(partial W_1)$],
    [$z_2 = W_2 a_1 + b_2$], [$(partial L)/(partial z_2)$ via chain rule],
    [$a_2 = sigma(z_2)$], [$(partial L)/(partial a_2) arrow.r (partial L)/(partial W_2)$],
    [$...$], [$...$],
    [$"output" = a_L$], [$(partial L)/(partial a_L) =$ loss gradient],
  )
]

#v(1em)

#text(size: 20pt, fill: rgb("#4DA6FF"))[
  Forward pass computes all $a_l$ and $z_l$. Backward pass computes all gradients in a single pass.
]

----

= Gradient Descent: Using the Gradients

#v(1em)

Once we have the gradients, we update each weight:

#align(center)[
  #text(size: 36pt)[
    $ W <- W - eta \. (partial L)/(partial W) $
  ]
]

#v(1em)

where $eta$ is the *learning rate* — a hyperparameter controlling step size.

#v(1em)

#align(center)[
  #table(
    columns: (1fr, 1fr, 1fr),
    stroke: none,
    inset: 8pt,
    align: center,
    [*Too large*], [*Too small*], [*Just right*],
    [Overshoots, oscillates], [Barely moves, slow], [Steady descent to minimum],
  )
]

#text(size: 18pt, fill: luma(160))[
  The learning rate is the most important hyperparameter — we'll explore convergence guarantees in the next lesson.
]

----

= Key Takeaways: Backpropagation

#v(1em)

+ Backpropagation = the chain rule applied systematically to computational graphs
+ Forward pass computes outputs; backward pass computes gradients
+ Each node needs only its *local gradient* — the graph handles the rest
+ Gradients flow backward through the same graph structure
+ Gradient descent uses these gradients: $W <- W - eta \. (partial L)/(partial W)$

#v(1em)

#text(size: 18pt, fill: luma(160))[
  Backpropagation scales to networks with billions of parameters — the same simple principle, applied at massive scale.
]

----

#align(center)[
  #v(3em)

  #text(size: 36pt)[Next: Convergence of Sequences]

  #v(1em)

  #text(size: 20pt, weight: "regular")[
    We've seen how iterative updates drive learning. \
    But when do these iterative processes actually \
    converge? That brings us to real analysis and \
    the theory of sequences.
  ]
]
