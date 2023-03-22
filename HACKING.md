# Notes about how things are done

## `failures` vector

The reason failures from cell checkers that can report multiple of
them are communicated via a mutable "out" vector is both that I expect
it to be the most efficient way to do it in the hot path (since the
vector can be reused), and that it seems simpler to write than using a
generator.
