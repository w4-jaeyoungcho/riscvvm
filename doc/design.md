## Plan

The components

* Lexer/Parser
* Architecture data
  * Documentation (limitations, extent)
* Assembler, scoped
  * Calc, eval
  * listing
* disasm

synchronuous VM
* VM
* scoped assembler
* rudimentary OS
* assembler producing ELF object
* vlang
* IO utils...

async VM

Start with lexer


Rust project layout
* src
  * lexer
  * parser
  * ...

### Lexer

Simply scheme-like language because parsing is bloody difficult

only ASCII

I will implement limited form of scheme lexer/parser
atoms and lists, no pairs

Might work on byte iterator

return token stream.

### Parser

Parser will read a determinate amount of tokens from token stream and produce an expr.

...

Actually this confuses me because I don't know behavior of streams or iterators.
Just block then, whatever.

### Architecture

Encoder and decoder
operate on exprs...


### Calc

evaluate arithmetic exprs

### ASM

ASM gets works on a ready series of exprs, per file.

levels of ASM

phase 1
* scanStatements - ordering of input - handle include commands
* see what it is
* collect symbol, location from labels

phase 2
* scanStatements
* see what it is - statements - directives
* handle asm directives
  * def
  * undef
  * equ
* evaluate symbols and arithmetic expressions
* encode

### Compiler

a plugin feature for ASM... I think that will make matters simpler.

ease routine routine definitions maybe...

### VM

machine
* cpu
  * register file
  * csr

peripherals
* memory - is memory peripheral?
* input
* output
