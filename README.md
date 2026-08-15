Dead simple ECTS calcualtor, using the formula:

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

https://github.com/user-attachments/assets/4f8ad77e-71f3-4889-b521-56c034bab153

## Installation

### Prerequisites
- Rust toolchain

### Install Locally
```sh
cd $HOME/Downloads
git clone https://github.com/nheggoe/ects-calc.git
cd ects-calc
cargo install --path .
```
