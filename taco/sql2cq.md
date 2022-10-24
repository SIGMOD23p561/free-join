The following SQL query (R has schema `(a,b)`, and S has schema `(y,z)`): 

```sql
SELECT a,b,z
  FROM R,S
 WHERE b=y
```

is translated to the following conjunctive query: 

```prolog
Q(a,b,z) :- R(a,b), S(b,z).
```

The algorithm is as follows: 

1. The `SELECT` clause simply becomes the head. 
2. Each relation in the `FROM` clause (e.g. `R` with schema `(a,b)`) becomes an atom (`R(a,b)`). Note that `S` becomes `S(y,z)` at this point, not `S(b,z)` yet. 
3. Create a [disjoint set](https://en.wikipedia.org/wiki/Disjoint-set_data_structure) data structure, with one (singleton) set per variable. For our example, we will have `{a},{b},{y},{z}`. 
4. For every equality constraint `b=y` in the `WHERE` clause, call `union` on the variables `b` and `y`. This will change our disjoint set into `{a},{b,y},{z}`. 
5. Iterate over all variables in the conjunctive query, and replace each variable `x` with `find(x)`. For our example, `find(a)=a` and `find(z)=z` because `{a}` and `{z}` are singleton sets. `find(y)` and `find(b)` may return either `y` or `b`, and either one is fine; if they both return `y`, the final CQ query will be the following (which is equivalent to the above): 

```prolog
Q(a,y,z) :- R(a,y), S(y,z).
```
