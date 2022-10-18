# Free Join

This is the artifact for SIGMOD 2023 paper #561. 
Due to time constraints, we provide instructions to run only one set of experiment. 

# Requirements

1. Standard linux tools, like `make`, `wget` etc. Please watch for errors and install any missing binaries. 
2. Rust, installed with https://rustup.rs
3. Any dependencies required to build [DuckDB](https://github.com/duckdb/duckdb#development). We include DuckDB's source code in this repository, with minimal modifiactions to produce machine-readable query plans. The Makefile builds DuckDB. 

# Running Free Join
Simply type `make` after installing the dependencies. 
The Makefile will Download datasets, compile the code, and run the experiment. 
The experiment output is plotted in `plot.html` (please open the file in a browser). 
It compares the performance of Free Join with DuckDB's binary hash join implementation. 

# The Code
Our Rust implementation of Free Join is found under `gj`. 
