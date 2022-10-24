```rust
// Q(x, y, z) :- R(x, y), S(y, z), T(x, z).

while !empty(R) && !empty(T) {
    let (Rx, Ra) = R[0]
    let (Tx, Ta) = T[0]
    let a = max(Rx, Tx)
    if a == Rx && a == Tx {
        let Ra = R[0].1
        let Ta = T[0].1
        while !empty(Ra) && !empty(S) {
            let b = max(Ra[0].0, S[0].0)
            if b == Ra[0].0 && b == S[0].0 {
                // let Rab = Ra[0].1
                let Sb = S[0].1
                while !empty(Sb) && !empty(Ta) {
                    let c = max(Sb[0].0, Ta[0].0)
                    if c == Sb[0].0 && c == Ta[0].0 {
                        // let Sbc = Sb[0].1
                        // let Tac = Ta[0].1
                        output(a, b, c);
                        Sb = Sb[1..]
                        Ta = Ta[1..]
                        continue
                    }
                    gallop(&Sb, c);
                    gallop(&Ta, c);
                }
                Ra = Ra[1..]
                S = S[1..]
                continue
            }
            gallop(&Ra, b)
            gallop(&S, b)
        }
        R = R[1..]
        S = S[1..]
        continue
    }
    gallop(&R, a);
    gallop(&T, a);
}
```

```rust
// Q(x, y, z) :- R(x, y), S(y, z), T(x, z).

while !empty(R) && !empty(T) {
    let (Rx, mut Ra) = R[0]
    let (Tx, mut Ta) = T[0]
    let a = max(Rx, Tx)
    if a == Rx && a == Tx {
        while !empty(Ra) && !empty(S) {
            let (Ray, Rab) = Ra[0]
            let (Sy, Sb) = S[0]
            let b = max(Ray, Sy)
            if b == Ray && b == Sy {
                while !empty(Sb) && !empty(Ta) {
                    let (Sbz, Sbc) = Sb[0]
                    let (Taz, Tac) = Ta[0]
                    let c = max(Sbz, Taz)
                    if c == Sbz && c == Taz {
                        output(a, b, c);
                        Sb = Sb[1..]
                        Ta = Ta[1..]
                        continue
                    }
                    gallop(&Sb, c);
                    gallop(&Ta, c);
                }
                Ra = Ra[1..]
                S = S[1..]
                continue
            }
            gallop(&Ra, b)
            gallop(&S, b)
        }
        R = R[1..]
        S = S[1..]
        continue
    }
    gallop(&R, a);
    gallop(&T, a);
}
```
