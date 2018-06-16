
use super::*;

use ::machine::*;
use ::machine::peri::*;

use ::asm;


#[test]
fn test_wfi() {
    let code = String::from(SYSTEM_HEADER) + "\
(wfi)
";

    let bin = asm::assemble_mem(&code).expect("assemble");

    let (mut m, output_device, input_device) = vpc(&bin, Vec::<u8>::new());

    match m.run(10, true) {
        Ok(cycles) => assert_eq!(cycles, 1),
        Err(e) => panic!("{:?}", e),
    }
}



#[test]
fn test_interrupt() {
    let code = String::from(SYSTEM_HEADER) + "\
(j RESET_HANDLER)
(nop)
(nop)
(nop)

(label TRAP_HANDLER)
; write to output
(lw t1 t3 0)
(sw t2 t1 0)

(lui t4 end_pc_target)
(addi t4 t4 end_pc_target)
(jalr zero t4 0)


(label RESET_HANDLER)
; enable global interrupt
(csrrsi zero 0x4 mstatus)
(li t0 0x800)
(csrrs zero t0 mie)

; t2 output
; t3 input
(lui t2 output)
(addi t2 t2 output)
(lui t3 input)
(addi t3 t3 input)

(wfi)
";

    let bin = asm::assemble_mem(&code).expect("assemble");

    let (mut m, output_device, input_device) = vpc(&bin, Vec::<u8>::new());

    // This should wfi
    match m.run(10, true) {
        Ok(cycles) => assert_eq!(cycles, 9),
        Err(e) => panic!("{:?}", e),
    }

    // Write to input
    input_device.borrow_mut().input("a");

    // This should terminate
    match m.run(10, true) {
        Ok(cycles) => panic!("unexpected wfi"),
        Err(RunError::Terminated) => (),
        Err(e) => panic!("{:?}", e),
    }

    let output = output_device.borrow().writer.clone();

    assert_eq!(output.len(), 1);
    assert_eq!(output[0], b'a');
}