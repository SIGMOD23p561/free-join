
Some Profiles contain the MARK Hash Join when the queries have IN clause.
Thus, we ignore this type of Join in our check.
And we also won't translate such join in generic join (Is it correct?)

(Except for the all hash join check, but it won't effect since MARK Join is implemented by Hash Join)

Moreover, we output the name of profiles containing MARK join into the stderr for further debugging.

If we only need to output the results of our checks, the following command is enough.

```shell
cargo run --package translator --bin translator ../logs/plan-profiles/ 1> output/res.txt
```