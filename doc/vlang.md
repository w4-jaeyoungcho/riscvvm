
Assembly language that can be directly expanded from higher level language.

Higher level language is assembly language augmented.

Target high level language semantics
* registrate for me
* function prologue, epilogue
* I want structured language. Away with LOOP_START, LOOP_END, LOOP_EXITs...
  * but no block... seems difficult

symbol - word, some have location
dereference follows as pointer

## Considerations
const arithmetic...


## Principles
functions are flat.
**all vars are word sized and reside in register file**
function args and return values better all be on register file
stack allocations - allocate on stack and set const to be offset

expressions translated as congruent chunks of machine code
only additional things added by compiler are function prologue and epilogue

assembly locations are pointer values

assembly compiler only has consts, not vars, to facilitate.

### inline assembly
use high-level feature as more robust assembly prelogue/epilogue macros 

since all vars are word sized and reside in register, can be readily referenced

; Just ref it
(addi n arg 1)

## Structured programming
provides boilerplate support with
* func
* loop
* if with begin

## Registration
..... I don't know how do this.

## Examples

### fib

```
; function definition
; function cannot jump outside nor can it be jumped in... let's have some structrure
(.func Fibonacci (n) (res)
    ; optional initializing things for variables come before statements
    (var not_used 7)
        
    (set res 1) ; watch out for garbage...
    
    (if (= n 0) (j RETURN))
    (if (= n 1) (j RETURN))
    (set res (+ (Fibonacci (- n 1)) (Fibonacci (- n 2))))
    
    ; RETURN is label implicitly set here why not 
)

; call from assembly
(li a0 7)
(call Fibonacci) 
(mv t1 a0)

; or in the case of fused assembly language
(set t1 (Fibonacci 7)) ; You better have saved all temp registers...
```

```
; stack allocation
(func stackering () ()
    ; initializing thing
    (const ARR_SIZE (words 10)) ; scoped const better come first to be referred in the rest of the function definition
    ; words multiplies by 4 why not
    (alloc arr ARR_SIZE) ; array of 10 words.
    (var i 0)
    
    (loop
        (if (= i ARR_SIZE) (j LOOP_EXIT)) ; LOOP_EXIT, LOOP_START, LOOP_END are implicitly set and scoped
        (set (@ (+ sp arr i)) i) ; pointer arithmetic with stack pointer to refer to stack allocations
        (set i (+ i 1))))
    
```

Ooh, I like this.



