# halo2 sample

## example 1

$$
a*b = c
$$

```rust
cargo run --bin factor
```

## example 2

$$
f(n+2) = f(n+1) + f(n) \\
f(0) = 1,\ f(1) = 1
$$

```rust
cargo run --bin fibonacci
```

## tutorial - Section 1

[halo2 tutorial](https://erroldrummond.gitbook.io/halo2-tutorial/)

$$
l \cdot s_l + r \cdot s_r + (l \cdot r) \cdot s_m + o \cdot s_o + s_c + PI = 0
$$

```rust
cargo run --bin section1
```
