Dead simple ECTS calculator, using the formula:

```math
\begin{aligned}
\text{ECTS weighted average}
&=
\frac{
\sum_{i=1}^{n}
\left(\text{grade points}_i \times \text{credits}_i\right)
}{
\sum_{i=1}^{n}\text{credits}_i
}
\\[1em]
&\mathrm{grade}_i \in \{A,B,\ldots,E\}
\\[0.5em]
&A \mapsto 5,\quad
B \mapsto 4,\quad
C \mapsto 3,\quad
D \mapsto 2,\quad
E \mapsto 1
\end{aligned}
```

https://github.com/user-attachments/assets/812bbd27-7365-4f0b-82c2-d9b4877e5453

## Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/)

### Build from source

Installed binary will have the name `ects-calc`

```sh
cargo install --git https://github.com/nheggoe/ects-calc.git
```
