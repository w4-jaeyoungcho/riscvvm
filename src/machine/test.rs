
use super::*;

use std;
use ::asm;

use ::machine::peri::*;

#[test]
fn test_func_stack() {
    let code = String::from(system_header) + "\
(lui sp initial_sp)
(addi sp sp initial_sp)

(jal ra (&- DO_THING pc))

(.begin
    (.def r_end t3)
    (lui r_end end_pc_target)
    (addi r_end r_end end_pc_target)

    (jalr zero r_end 0)
)

(: DO_THING)
(.func
    (.stack ARR1 (*4 3))
    (.stack ARR2 (*4 3))

    (.begin
        (.def i t1)

        ; arr1 <- [1, 2, 3]
        (li i 1)

        (sw sp i (+ ARR1 0))
        (addi i i 1)
        (sw sp i (+ ARR1 4))
        (addi i i 1)
        (sw sp i (+ ARR1 8))

        ; MOVEW(&mut arr2, &arr1, 3)
        (addi a0 sp ARR2)
        (addi a1 sp ARR1)
        (li a2 3)
        (jal ra (&- MOVEW pc))
    )

    ; output arr2
    (.begin
        (.def temp t0)
        (.def r_output t3)

        (lui r_output output)
        (addi r_output r_output output)
        (lw temp sp (+ ARR2 0))
        (sw r_output temp 0)
        (lw temp sp (+ ARR2 4))
        (sw r_output temp 0)
        (lw temp sp (+ ARR2 8))
        (sw r_output temp 0)
    )
)


(: MOVEW) ; (&mut, &, number of words)
(.leaf
    (.def dest a0)
    (.def src a1)
    (.def n a2)
    (.def temp t0)
    (.def dest_end t1)

    (slli n n 2)
    (add dest_end dest n)

    (.block LOOP
        (beq dest dest_end (&- LOOP.END pc))
        (lw temp src 0)
        (sw dest temp 0)

        (addi src src 4)
        (addi dest dest 4)
        (j (&- LOOP.BEGIN pc))
    )
)


";

    let output = test_for_output(&code, true);

    assert_eq!(output.len(), 3, "length");
    assert_eq!(&output, &[1u8, 2, 3], "value");
}

#[test]
fn test_funct() {
    let code = String::from(system_header) + "\

(lui sp initial_sp)
(addi sp sp initial_sp)

; call fibonacci(5)
(.begin
    (li a0 5)
    (jal ra (&- fibonacci pc)))

; write
(.begin
    (.def r_output t3)
    (lui r_output output)
    (addi r_output r_output output)

    (sw r_output a0 0))

; exit
(.begin
    (.def r_end t3)
    (lui r_end end_pc_target)
    (addi r_end r_end end_pc_target)

    (jalr zero r_end 0))

(: fibonacci)
(.func
    (.var n)
    (.var saved)
    (.temp unused)

    (mv n a0)

    (li a0 1)
    (beq n zero (&- RETURN pc))
    (beq n a0 (&- RETURN pc))

    (addi a0 n -1)
    (jal ra (&- fibonacci pc))
    (mv saved a0)

    (addi a0 n -2)
    (jal ra (&- fibonacci pc))
    (add a0 saved a0))
";


    let output = test_for_output(&code, true);

    assert_eq!(output.len(), 1, "length");
    assert_eq!(output[0], 8, "value");
}


#[test]
fn test_begin_as_loop() {
    let code = String::from(system_header) + "\
(.begin
    (.def r_output t3)

    (lui r_output output)
    (addi r_output r_output output)

    (.def i t1)
    (.def num10 t2)

    (li i 0)
    (li num10 10)

    (.block LOOP
        (beq i num10 (&- LOOP.END pc))
        (sw r_output i 0)
        (addi i i 1)
        (j (&- LOOP.BEGIN pc))))

(.begin
    (.def r_end t3)
    (lui r_end end_pc_target)
    (addi r_end r_end end_pc_target)

    (jalr zero r_end 0))
";

    let output = test_for_output(&code, true);

    assert_eq!(output.len(), 10, "length");
    for i in 0..10 {
        assert_eq!(output[i] as i32, i as i32, "value");
    }
}

#[test]
fn test_begin() {
    let code = String::from(system_header) + "\
(.begin
    (.def r_output t3)
    (.def r_value t1)

    (lui r_output output)
    (addi r_output r_output output)

    (li r_value 7)
    (sw r_output r_value 0))

(.begin
    (.def r_end t3)
    (lui r_end end_pc_target)
    (addi r_end r_end end_pc_target)

    (jalr zero r_end 0))
";

    let output = test_for_output(&code, true);

    assert_eq!(output.len(), 1, "length");
    assert_eq!(output[0], 7, "value");
}

#[test]
fn test_wfi() {
    let code = String::from(system_header) + "\
";
}

#[test]
fn test_sum() {

    let code = String::from(system_header) + "\
; add 1 to 10 on memory[0x10000]
; t3 output
; t4 input
(lui t2 output)
(addi t2 t2 output)
(lui t3 input)
(addi t3 t3 input)

; t0 = iter
; t1 = sum
; t4 = 11
(li t0 1)
(li t1 0)
(li t4 11)

(: LOOP_START)
(add t1 t1 t0)
(addi t0 t0 1)
(bne t0 t4 (&- LOOP_START pc))

; output
(sw t2 t1 0)
; done
; t0 = end_pc_target
(lui t0 end_pc_target)
(addi t0 t0 end_pc_target)
(jalr zero t0 0)
(lw t1 zero 0)
";

    let output = test_for_output(&code, true);

    // Check output from output device
    assert_eq!(output.len(), 1, "output len");
    assert_eq!(output[0], 55, "output value");
}

#[test]
fn test_exception() {
    let code = String::from(system_header) + "\
; write 0x01, trigger trap, write 0x02, terminate
(j RESET_HANDLER)
(nop)
(nop)
(nop)

(: TRAP_VECTOR)
; write 0x02
(li t1 0x02)
(sw t3 t1 0)
; done
(lui t4 end_pc_target)
(addi t4 t4 end_pc_target)
(jalr zero t4 0)

(: RESET_HANDLER)
; t3 = output
(lui t3 output)
(addi t3 t3 output)

(li t1 0x01)
(sw t3 t1 0)
; trigger exception
; LOAD_ACCESS_MISALIGNED
(lw zero zero 0x2)
";
    let output = test_for_output(&code, false);

    assert_eq!(&output, &[1u8, 2u8]);
}

const system_header: &'static str = "\
; peripherals
(.equ output 0x00100000)
(.equ input 0x00100004)
(.equ end_pc_target 0x10000000)
(.equ mtvec 0x00000010)

; convention
(.equ initial_sp 0x00010000)
";


use std::rc::Rc;
use std::cell::RefCell;

struct VecPtr(Rc<RefCell<Vec<u8>>>);

impl VecPtr {
    fn new() -> VecPtr {
        VecPtr(Rc::new(RefCell::new(Vec::<u8>::new())))
    }

    fn clone(&self) -> VecPtr {
        VecPtr(self.0.clone())
    }
}

impl Write for VecPtr {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.0.borrow_mut().write(buf)
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.0.borrow_mut().flush()
    }
}

// Helper for testing memory and stuff.. for now things that don't involve IO
// Also things that don't involve exception
fn test_for_output(code: &str, no_exception: bool) -> Vec<u8> {
    let bin = match asm::assemble_mem(&code) {
        Ok(v) => v,
        Err(e) => {
            // Print the line as well as the error
            let line = code.lines().nth(e.line_index).unwrap();
            panic!("Assemble error: {:?}; line: {}", e, line);
        }
    };

    // Print disassembled code
    {
        let reader = &bin[..];

        let mut writer = Vec::<u8>::new();

        ::disasm::disassemble(&mut writer, reader).expect("disassemble");

        let disassembled = std::str::from_utf8(&writer[..]).unwrap();

        println!("Disassembled:");
        println!("{}", disassembled);
    }


    // Machine

    const MEMORY_START: u32 = 0;
    const MEMORY_WIDTH: u8 =16;
    const MEMORY_PROBE: u32 = 0x1000;

    const OUTPUT_START: u32 = 0x100000;

    const TICK_LIMIT: u32 = 500; // Adjust

    let writer = VecPtr::new();

    let mut memory = Memory::new(Some(MEMORY_PROBE));
    let mut output_device = OutputDevice::new(writer.clone());

    {

        let mut m = Machine::new();
        m.cpu.pc_trail = Some(Vec::new());

        memory.load(&bin[..]);
        m.attach("memory", memory, MEMORY_START, MEMORY_WIDTH);
        m.attach("output", output_device, OUTPUT_START, 0);
        match m.run(TICK_LIMIT, no_exception) {
            Ok(_) => (),
            Err(e) => {

                match e {
                    RunError::Exception(cause) => panic!("test_run encountered exception at {:X}, cause: {}", m.cpu.epc, m.cpu.cause),
                    RunError::CyclesLimitExceeded(last_pc) => panic!("test_run exceeded tick limit of {}, last_pc={:08X}", TICK_LIMIT, last_pc),
                    RunError::Terminated => (),
                }
            }
        }
    }

    let output = writer.0.borrow().clone();
    output
}
#[test]
fn test_set_patch() {
    let base = 0x12345678;
    let expected = 0x123FED78;
    let result = set_patch(base, 8, 12, 0xFED);
    assert_eq!(result, expected);
}