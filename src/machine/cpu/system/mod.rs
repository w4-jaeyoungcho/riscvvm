
use super::*;
use ::arch::system::*;

// Interrupt, exception

pub const MTVEC_VALUE: u32 = 0x00000010;

pub fn is_interrupt_possible(cpu: &Cpu, to: u8) -> bool {
    if to > cpu.level {
        return true;
    }
    if to < cpu.level {
        return false;
    }
    return (cpu.status >> (IE_BASE + cpu.level)) & 0x1 == 1
}

pub fn interrupt(cpu: &mut Cpu, to: u8, cause: u32) {
    cpu.cause = cause | (1u32 << INTERRUPT);

    cpu.trap(to);
}

// Currently, to is Machine
pub fn trap(cpu: &mut Cpu, to: u8) {
    assert!(to >= cpu.level);

    /*
    mstatus[toIE] <- 0

    mstatus[toPP] <- prev level
    mstatus[toPIE] <- mstatus[prevIE]
    level <- to level

    mepc <- pc + 4
    pc <- trap vector
    */

    cpu.status &= !(1u32 << (IE_BASE + to));

    if to == USER {
        // nop
    } else if to == SUPERVISOR {
        cpu.status &= !(1u32 << SPP);
        cpu.status |= (cpu.level as u32) << SPP;
    } else if to == MACHINE {
        cpu.status &= !(3u32 << MPP);
        cpu.status |= (cpu.level as u32) << MPP;
    }

    let prev_ie = (cpu.status >> (IE_BASE + cpu.level)) & 0x1;

    cpu.status &= !(1u32 << (PIE_BASE + cpu.level));
    cpu.status |= prev_ie << (PIE_BASE + cpu.level);

    cpu.level = to;

    cpu.epc = cpu.pc;

    cpu.pc = cpu.mtvec & !0x3;
}

pub fn exception(cpu: &mut Cpu, cause: u32) {
    debug!("exception: cause={:08X} pc={:08X}", cause, cpu.pc);

    cpu.cause = cause;
    cpu.trap(MACHINE);
}

// from can't be higher than the current level
pub fn ret(cpu: &mut Cpu, from: u8) {
    /*
    prev <- mstatus[from's PP]
    prevIE <- mstatus[from's PIE]
    mstatus[prev's IE] <- prevIE
    Level <- prev

    mstatus[from's PIE] <- 1
    mstatus[from's PP] <- User, or Machine if User is not supported. Note WLRL

    pc <- mepc
    */

    // Get prev fromPP and replace with USER
    let prev: u8;
    if from == USER {
        prev = USER;
    } else if from == SUPERVISOR {
        let pp = patch::Patch { offset: SPP, length: 1 };
        let p = pp.exchange_on_ref(&mut cpu.status, 0);
        prev = p as u8;
    } else if from == MACHINE {
        let pp = patch::Patch { offset: MPP, length: 2 };
        let p = pp.exchange_on_ref(&mut cpu.status, 0);
        prev = p as u8;
    } else {
        panic!("from: {}", from)
    }

    assert!(prev == USER || prev == SUPERVISOR || prev == MACHINE);

    // prevIE <- mstatus[from's PIE]
    let prev_ie = (cpu.status >> (PIE_BASE + from)) & 0x1;

    // mstatus[prev's IE] <- prevIE
    let prev_ie_patch = patch::Patch { offset: IE_BASE + prev, length: 1 };
    prev_ie_patch.write_on_ref(&mut cpu.status, prev_ie);

    // Level <- prev
    cpu.level = prev;

    // mstatus[from's PIE] <- 1
    let from_pie_patch = patch::Patch { offset: PIE_BASE + from, length: 1 };
    from_pie_patch.write_on_ref(&mut cpu.status, 1);

    // pc <- mepc
    cpu.pc = cpu.epc;
}

// CSRs

// Accessibility are checked from cycle

pub fn read_csr(cpu: &mut Cpu, csr: u32) -> Result<u32, ()> {
    let csr = match CSR::from_u32(csr) {
        Some(csr) => csr,
        None => return Err(()),
    };

    let value = match csr {
        CSR::MCYCLE => cpu.cycle as u32,
        CSR::MCYCLEH => (cpu.cycle >> 32) as u32,

        // Machine specs
        //TODO

        CSR::MSTATUS => cpu.status,

        // Exceptions
        CSR::MIP => cpu.ip,
        CSR::MIE => cpu.ie,

        _ => panic!("implement me"),
    };
    Ok(value)
}

pub fn write_csr(cpu: &mut Cpu, csr: u32, v: u32) -> Result<(), ()> {
    let csr = match CSR::from_u32(csr) {
        Some(csr) => csr,
        None => return Err(()),
    };

    match csr {
        CSR::MSTATUS => cpu.status = v,
        CSR::MIP => panic!("implement me!"),
        CSR::MIE => cpu.ie = v,
        _ => panic!("implement me!"),
    }

    Ok(())
}

pub fn csr_level(csr: u32) -> u8 {
    ((csr >> 8) & 0x3) as u8
}

pub fn is_csr_readonly(csr: u32) -> bool {
    let access_bits = (csr >> 10) & 0x3;
    access_bits == 11
}
