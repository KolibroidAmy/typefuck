A compile-time brainfuck interpreter, written using rust's type system.

I've provided a macro to make creating the program/input easier, but the interpreter uses types and traits only.

Output is provided by intentionally causing a type error that contains final output type, so you never even have to run the binary!
Just like the code itself, it's output is very hard to read (specifically its a pair containing an upside-down stack of natural numbers implemented using a cons list :) ).

# Why
Honestly probably a mental health issue.

# How
Natural numbers and cons lists are pretty self-explanatory.

Tapes are implemented using a left and right stack (cons list), with the head being the top of the right stack.

The interpreter generates its next state based on its state and its various tapes, and does so recursively until reaching a halting state.

Input and output is also implemented using tapes.
