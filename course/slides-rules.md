# Best Practices for STEM Slide Design & Narration

A practical guide for building effective slides and writing narration scripts for STEM courses, grounded in cognitive science research.

---

## 1. Cognitive Science Foundations

Based on Richard Mayer's *Multimedia Learning Theory* and Garr Reynolds' *Presentation Zen*.

### 1.1 Three Core Findings

1. **Dual-Channel Processing** — People have separate channels for visual and verbal information. Slides should leverage both: visuals on screen, words spoken by the narrator.
2. **Limited Capacity** — Each channel can process only a few pieces of information at a time. Overloading a slide with text, equations, and diagrams simultaneously overwhelms the learner.
3. **Active Processing** — Understanding happens when learners pay attention to relevant material, organize it into a coherent structure, and integrate it with prior knowledge. Slides must guide this process, not dump raw information.

### 1.2 Mayer's Evidence-Based Principles

| Principle | Rule | Evidence |
|-----------|------|----------|
| **Multimedia Effect** | Narration + relevant visuals outperform narration alone. | Adding diagrams to spoken explanation improves retention by ~50%. |
| **Modality Principle** | Explain visuals with audio narration, not on-screen text. | The visual channel is less overloaded when text is removed from the slide. |
| **Redundancy Principle** | Do NOT display the full narration text on screen. | Reading and hearing the same words simultaneously reduces learning. |
| **Coherence Principle** | Remove extraneous decoration, clip art, and tangential detail. | Decorative elements compete for cognitive resources. |
| **Signaling Principle** | Use visual cues (arrows, highlights, color) to show what to focus on. | Learners process faster when attention is guided. |
| **Segmenting Principle** | Break complex topics into small, learner-paced segments. | One concept per slide; pause between major ideas. |
| **Temporal Contiguity** | Place corresponding words and pictures near each other in time. | Narration should describe a visual at the moment it appears, not before or after. |
| **Personalization Principle** | Use conversational language, not formal detached prose. | First person ("we", "let's") increases engagement without reducing rigor. |

### 1.3 Practical Implication

> Can your slide be understood in 3 seconds of scanning? If not, redesign it.

Every element on a slide must earn its place. If it does not directly support the learning objective, remove it.

---

## 2. Slide Design for STEM Content

### 2.1 One Concept Per Slide

A slide is a single unit of thought. Do not combine unrelated ideas.

**Bad:** A single slide covering "derivatives, integrals, and the fundamental theorem of calculus."
**Good:** Three slides, each covering one topic, each building on the previous.

### 2.2 Progressive Disclosure

Build complex ideas incrementally across multiple slides. Never present a finished, dense result all at once.

**Example — Introducing a neural network layer:**
- Slide 1: Show a single neuron with inputs and weights.
- Slide 2: Add the activation function.
- Slide 3: Show multiple neurons forming a layer.
- Slide 4: Stack layers into a network.

This lets learners construct understanding step by step instead of decoding a complex diagram.

### 2.3 Visual Hierarchy

Every slide should have a clear reading order:

1. **Title** — States the concept (e.g., "The Chain Rule").
2. **Primary visual** — The key equation, diagram, or code snippet. Large, centered.
3. **Annotations** — Brief labels, arrows, or color highlights on the primary visual.
4. **Optional note** — A single line of context at the bottom, in smaller text.

### 2.4 Equations

- **Display one key equation per slide.** Derivations can span multiple slides.
- **Use color to highlight terms** that change or matter most (e.g., highlight the learning rate in a gradient update).
- **Never cram multiple numbered equations on one slide.** If you need to reference a previous result, put it in a small sidebar with "(from slide 14)" rather than repeating the full derivation.
- **Use consistent notation** across all slides. Define symbols once, then reuse.

### 2.5 Diagrams and Figures

- **Annotate directly on the diagram**, not in a separate legend below.
- **Use arrows and labels** to show flow, direction, or causality.
- **Simplify.** Remove grid lines, axis numbers, and decorative elements unless they serve the explanation.
- **Prefer vector graphics** (SVG) over raster images for diagrams — they scale cleanly at any zoom.

### 2.6 Code Listings

- **Show only the relevant lines.** Use `...` or `// rest omitted` for boilerplate.
- **Highlight the lines being discussed** with a background color.
- **Use a monospace font with syntax highlighting** in a consistent color scheme.
- **Never show more than 15 lines of code per slide.**

### 2.7 Data Visualizations (Charts, Graphs)

- **One message per chart.** If the chart tells multiple stories, split it into multiple slides.
- **Label axes clearly.** Include units.
- **Use color to encode one variable.** Do not use color for decoration.
- **State the takeaway** in the slide title (e.g., "Loss decreases exponentially after epoch 10" rather than "Training Loss Chart").

### 2.8 Color and Typography

- **Dark background with light text** or **light background with dark text** — pick one and be consistent.
- **Maximum 2 fonts:** one for titles, one for body/code.
- **Use color sparingly:** one accent color for emphasis, a secondary color for highlights. Avoid rainbow palettes.
- **High contrast** for all text and equations — never gray on gray.

### 2.9 Whitespace

Whitespace is not wasted space — it reduces cognitive load. Leave generous margins. Do not fill every corner of the slide.

---

## 3. Narration Script Guidelines

### 3.1 What the Narrator Says vs. What Appears on Screen

The narration and the slide are **complementary, not redundant.**

| On the slide | In the narration |
|-------------|-----------------|
| Key equation | Derivation intuition and why it matters |
| Diagram | Step-by-step walkthrough of what each part represents |
| Code snippet | What the code does conceptually, not a line-by-line reading |
| Definition | Example, motivation, or edge case |
| Result / theorem | Proof sketch or intuition behind why it's true |
| Plot / chart | The trend or anomaly the learner should notice |

**Never read the slide text aloud.** The narrator should add value beyond what the learner can read themselves.

### 3.2 Per-Slide Script Structure

For each slide, the narration should follow this arc:

1. **Hook (5-10 seconds)** — Connect to prior knowledge or pose a question.
   - "Last time we saw that gradients point uphill. What if we want to go downhill?"
   - "We've been working with vectors. But what happens when we need to compare their directions?"

2. **Explain (20-60 seconds)** — Walk through the slide content, adding context and intuition.
   - Describe visual elements using spatial language ("on the left", "the blue curve").
   - Translate equations into plain language before diving into symbols.
   - Give a concrete example before stating the general rule.

3. **Connect (5-10 seconds)** — Link back to the bigger picture or foreshadow what comes next.
   - "This is why the learning rate matters — we'll see the consequences next."
   - "This identity is the backbone of everything we do in optimization."

4. **Transition (3-5 seconds)** — A brief bridge to the next slide.
   - "Let's see this in action with a real example."
   - "Now that we have the definition, let's prove it."

### 3.3 Timing Guidelines

| Slide type | Recommended duration |
|-----------|---------------------|
| Title / section divider | 5-10 seconds |
| Definition or theorem statement | 15-25 seconds |
| Worked example / derivation step | 30-60 seconds |
| Diagram walkthrough | 30-45 seconds |
| Code explanation | 30-60 seconds |
| Summary / recap | 15-30 seconds |

**Average pace:** 30-45 seconds per slide. Faster than 15 seconds means the slide is unnecessary; slower than 90 seconds means the slide needs to be split.

### 3.4 Pacing for Equations and Proofs

- **Pause 1-2 seconds** before stating a key result — let the learner see it on screen before hearing about it.
- **Read equations in plain language first**, then in mathematical notation.
  - Say: "the gradient of the loss with respect to the weights" before saying "partial L, partial w."
- **For multi-step derivations:** one step per slide, with narration explaining the *why* of each step, not just the mechanical manipulation.
- **Slow down at inflection points** — the moment a non-obvious trick or substitution is applied deserves extra time.

### 3.5 Handling Diagrams

- **Describe spatial layout first:** "In this diagram, the input layer is on the left, the hidden layer in the middle, and the output on the right."
- **Then describe the process:** "Data flows from left to right through these connections."
- **Use the pointer/cursor** to trace paths while narrating — the visual motion reinforces the verbal explanation.
- **Call out details selectively** — do not describe every arrow. Highlight the ones that matter for the current point.

### 3.6 Signposting Language

Use structural cues so learners always know where they are:

- **Starting a section:** "We're now moving into [topic]. This is the third of five parts."
- **Within a derivation:** "Step one: we substitute. Step two: we simplify."
- **Summarizing:** "So far we've established three things: ..."
- **Transitioning:** "That covers [topic A]. Now let's turn to [topic B]."
- **Concluding a lecture:** "Today we learned [X]. Next time we'll extend this to [Y]."

### 3.7 Language and Tone

- **Use "we" and "let's"** instead of "you should" or "one must." This creates a collaborative feel.
- **Acknowledge difficulty:** "This step is tricky — let's slow down and see why it works."
- **Ask rhetorical questions** to maintain engagement: "Why does this converge? Let's look at the Lyapunov function."
- **Avoid filler phrases:** "As you can see," "Obviously," "It's clear that." These signal laziness, not clarity.

---

## 4. Common Anti-Patterns to Avoid

| Anti-pattern | Why it fails | Fix |
|-------------|-------------|-----|
| Walls of text | Overloads the visual channel; learner reads instead of listening | Replace 80% of text with visuals; move detail to narration |
| Reading slides aloud | Redundant; wastes the audio channel | Narration adds context, not repetition |
| Wall of equations | No entry point; learners cannot parse the structure | One equation per slide; build sequentially |
| Decorative clip art | Violates coherence principle; adds cognitive noise | Remove entirely or replace with meaningful visuals |
| Inconsistent notation | Forces learners to re-decode symbols on every slide | Define once; use a notation reference slide |
| No pauses between sections | Learners cannot consolidate before new material arrives | Add section divider slides with 5-10 seconds of silence |
| Fast-forwarding through proofs | Proofs are where understanding is constructed | One logical step per slide; narrate the reasoning |

---

## 5. Slide Types Catalog for STEM Courses

### 5.1 Standard Slide Types

| Type | Purpose | Content on slide | Narration focus |
|------|---------|-----------------|----------------|
| **Title** | Open a new section | Section name + number | Brief overview of what's coming |
| **Definition** | Introduce formal concept | Term + precise statement | Motivation: why do we define this? |
| **Theorem** | State a key result | Theorem name + statement | Intuition: what does it mean in plain language? |
| **Proof step** | Build a derivation | One step, highlighted | Why this manipulation? What's the insight? |
| **Diagram** | Visual explanation | Clean annotated figure | Spatial walkthrough, then process flow |
| **Example** | Concrete instance | Setup + partial solution | How to approach it; where to apply the method |
| **Code** | Implementation | Relevant lines, highlighted | Conceptual meaning, not line-by-line reading |
| **Comparison** | Contrast two approaches | Side-by-side layout | When to use which; trade-offs |
| **Summary** | Recap key points | Bullet list (max 5 items) | Emphasize the most important takeaway |
| **Exercise** | Prompt practice | Problem statement | Hints or suggested approach |

### 5.2 Recommended Sequence per Topic

A typical topic follows this arc:

1. **Motivation slide** — Why does this matter? What problem does it solve?
2. **Definition slide** — Formal statement.
3. **Intuition slide** — Diagram, analogy, or concrete example.
4. **Derivation slides** (1-5 slides) — Step-by-step, progressive disclosure.
5. **Result slide** — The final theorem or formula, cleanly presented.
6. **Application slide** — How is this used in practice?
7. **Summary slide** — Key takeaway in one sentence.

---

## 6. VecSlide-Specific Recommendations

### 6.1 Audio-Visual Synchronization

In the `.vecslide` format, audio (Opus in `.ogg`) is synchronized with slides via absolute timestamps in the manifest. Best practices:

- **Mark timestamps at natural speech boundaries**, not mid-sentence.
- **Leave 1-2 seconds of silence** at section boundaries for the learner to consolidate.
- **Align visual transitions with narration transitions** — advance the slide exactly when the narrator says the transition phrase.

### 6.2 Pointer / Cursor Usage

VecSlide renders an invisible pointer that becomes visible only during movement, with a fading SVG trail.

- **Use the pointer to trace paths** in diagrams (e.g., following a gradient descent trajectory on a contour plot).
- **Point to specific terms** in equations as they are being explained.
- **Circle or underline** key results before stating them verbally.
- **Do not leave the pointer moving constantly** — it should animate deliberately, then disappear when idle.

### 6.3 Slide Timing

- **Precompute slide durations** from the narration script before recording.
- **Add buffer** — each slide should have 0.5-1 second of visual-only time before narration begins, giving learners time to scan the slide.
- **Long slides (>60s)** should be split into multiple slides with identical visuals but different highlighted regions, to maintain the signaling principle.

---

## Sources

- Garr Reynolds, *Presentation Zen* — http://www.presentationzen.com/
- Richard E. Mayer, *Multimedia Learning* (Cambridge University Press, 2009)
- Chris Anderson, "How to Give a Killer Presentation," *Harvard Business Review* (2013)
- Edward Tufte, *The Visual Display of Quantitative Information*
- Jean-luc Doumont, *Trees, Maps, and Theorems* — effective communication for engineers and scientists
