# nano-arena
[![Build Status](https://travis-ci.org/bennetthardwick/nano-arena.svg?branch=master)](https://travis-ci.org/bennetthardwick/nano-arena)

A tiny arena allocator that uses atomics in keys and supports split mutable borrows.

## Features
- Constant time allocations and removals
- Split mutable borrows inside arena
- Iter methods
- Easily convert Vec <-> Arena
- Easy trees and graphs with cyclic references
