
Based on plan of vlang

Assembler const.
(const BUF_SIZE (words 10))

No def, undef. Will use registration of vlang

(.rodata)
(: MY_NUMS)
(.word 0x11 0x12 9)

(.data)
(: MY_BUF)
(.bytes (words 10))

## Nicities
leaf function doesn't save ra. also documenting effect that it is a leaf function.
.leaf


## Namespace. Scoped Assembly
scoping... so that I can have scoped local definitions and labels

test_sum written in enhanced vasm

```
(.equ OUTPUT_ADDR 0x00100000)
(.equ INPUT_ADDR 0x00100004)

(.begin
    ; give registers names, in this scope. They are now aliases.
    (.def output t2)
    (.def input t3)

    (set t2 OUTPUT_ADDR) ; in proper asm, this is called just li
    (set t3 INPUT_ADDR)
    
    (set t0 1)
    (set t1 0)
    (set t4 11)
    
    (.begin
        (add t1 t1 t0)
        (addi t0 t0 1)
        (bne t0 t4 (&- BEGIN_START pc)))
     
    (sw output t1 0)
    
    ; done
    (def t0 end_pc_target) ; def at the middle
    (set end_pc_target END_PC_TARGET)
    (jalr zero end_pc_target 0)
    (lw t1 zero 0))
    
```

## Function call
stack moves by the multple of 8 bytes... ok.

Prologue

stack structure
* (where temporaries will be saved) not used for now
* evicted saved register values
* allocated stack space

save ra and other evicted saved registers
adjust sp

Epilogue
revert sp
restore all evicted saved registers
jump to ra

## Function macros

```
; like .begin but does function macros - saves and restores saved registers that are used
(: fibonacci)
(.func
    (.var n a0) ; allocate saved register. as convenience specify register to set initial value
    (.var saved)
    (.temp unused) ; allocate temp register - make sure to not use argument registers...
    
    (li a0 1) ; a0 as return value register
    (beq n 0 (&- RETURN pc))
    (beq n 1 (&- RETURN pc))
    
    (addi a0 n -1)
    (jal ra (&- fibonacci pc))
    (mv saved1 a0)
    
    (addi a0 n -2)
    (jal ra (&- fibonacci pc))
    (add a0 saved1 a0))  
```

## Stack management
Might as well leave stack management to the assembler.
I might need stack allocation for more than saving saved registers

```
(: stackering)
(.func
    (.stack arr1 (* 10 4)) ; arr1 as stack offset is available as local compiler symbol
    (.stack arr2 (* 20 4))
    
    (sw sp a0 arr1)) ; using the stack offset
```
## Assembler Arithmetic
arithmetic works on sign extended i64 type

complicated because vasm language is neither c nor scheme...

I'll do c. Because the scheme things seem weird. (logand, logior, ...)

unary ops
- (? dang) - subsumed to the additive -
^

multiplicative
vararg
*
/
%
&^
&

non vararg
<<
>>>
>>


additive vararg
+
-
|
^ : why is XOR additive?

equational
following c (go)
== != < <= >=

logical multiplicative vararg
&&

logical additive vararg
||

### Design
eval takes symbols and type extended for functions... why?
Why not.

