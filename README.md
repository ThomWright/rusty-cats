# Rusty CATS

[Another go at this](https://github.com/ThomWright/cats)

## Goal

A quantitative measure of coupling and cohesion between TypeScript modules/files.

As I see things, there are two main structures to consider here:

1. the dependencies between modules/files (a directed graph)
2. the file/directory structure (a tree)

Some things I think are good:

- the dependency graph being acyclic
- the dependency graph being tree-like
- the dependency graph closely mirroring the file/directory structure
- dependencies being close together (in terms of how far you need to traverse the directory tree to resolve a dependency)

I am not sure how best to quantify any of these measures.

## An information theory approach

This approach is [described here](http://www.sdml.cs.kent.edu/library/Allen99.pdf).

Pros:

- gives quantitative measures for:
  - intermodule coupling
  - intramodule coupling
  - cohesion

Cons:

- I'm not familiar enough with the maths to quickly understand or apply it
- assumes all modules are equal, does not consider submodules
  - e.g. if a file is a node, and a directory is a grouping of nodes (a module), how do we handle subdirectories?

### Definitions

- _MS_ - Modular system, represented as a graph
- _S_ - a subgraph with n+1 nodes, including 1 for the environment (disconnected)
- _n<sub>s</sub>_ - number of distinct labels (each node is labelled with the set of connected edges)
- _p<sub>l</sub>_ - proportions of distinct labels
- _p<sub>L(i)</sub>_ - proportion of a node _i_'s distinct label set (relative to total number of nodes)
- [_Entropy_](https://en.wikipedia.org/wiki/Entropy_(information_theory)) - average information per node
- _Minimum description length_ - the total
amount of information in the structure of the graph
- _I(S)_ - minimum description length
- _Intermodule coupling_ - minimum description length of the relationships in _S_ where _S_ is a subgraph with intermodule edges only
- _Intramodule coupling_ - minimum description length of the relationships in _S'_ where _S'_ is a subgraph with intramodule edges only
- _Cohesion_ - intramodule coupling / maximum intramodule coupling (all nodes connected)

### Equations

#### Entropy of the distribution of node labels

_H(S) = Σ(-p<sub>l</sub> log p<sub>l</sub>)_
<br>from _l=1_ to _n<sub>s</sub>_

_H(S) = Σ((-log p<sub>L(i)</sub>)/(n + 1))_
<br>from _i=0_ to _n_

#### Minimum description length

_I(S) = (n + 1) H(S)_

_I(S) = Σ(-log p<sub>L(i)</sub>)_
<br>from _i=0_ to _n_

### Example

All diagrams from [1].

![Module diagram](./docs/module-diagram.png)

#### Intermodule coupling

This is an example of intermodule coupling. All intramodule edges have been removed.

![Intermodule coupling diagram](./docs/intermodule-coupling-diagram.png)

![Intermodule coupling table](./docs/intermodule-coupling-table.png)

Node 3 has _p<sub>L(i)</sub>_ of 0.467 because there are 7 nodes with the same distinct label set, and 15 nodes total: 7/15 = 0.467.

| Node | 1 | 2 | 3 | 4 | 7 | 11 | p<sub>L(i)</sub> |
|------|---|---|---|---|---|----|------------------|
| 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |
| 1 | 1 | 1 | 1 | 0 | 0 | 0 | 0.067 |
| 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0.067 |
| 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |
| 4 | 0 | 0 | 0 | 0 | 1 | 0 | 0.067 |
| 5 | 0 | 1 | 0 | 0 | 1 | 0 | 0.067 |
| 6 | 0 | 0 | 1 | 0 | 0 | 0 | 0.067 |
| 7 | 0 | 0 | 0 | 0 | 0 | 1 | 0.133 |
| 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |
| 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |
| 10 | 0 | 0 | 0 | 0 | 0 | 1 | 0.133 |
| 11 | 0 | 0 | 0 | 1 | 0 | 0 | 0.067 |
| 12 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |
| 13 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |
| 14 | 0 | 0 | 0 | 0 | 0 | 0 | 0.467 |

Using the equation for entropy:

_H(S) = 7\*0.0732 + 6\*0.26 + 2\*0.194 = 2.46 bits per node_

And minimum description length:

_I(S) = 15 * 2.46 = 36.9 bits_

## Reference material

- [1] [Measuring Coupling and Cohesion: An Information-Theory Approach](http://www.sdml.cs.kent.edu/library/Allen99.pdf)
- [2] [Experiments with Coupling and Cohesion Metrics in a Large System](http://www.csi.uottawa.ca/~tcl/papers/metrics/ExpWithCouplingCohesion.pdf)
- [3] [Quantitative models of cohesion and coupling in software](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.94.183&rep=rep1&type=pdf)
