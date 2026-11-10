use bitbybit::{bitenum, bitfield};
use arbitrary_int::{u22, u2, u3};
#[bitfield(u22, default=0,forbid_overlaps, debug)]
pub struct Control {
    #[bit(21, rw)]
    reg_we: bool,
    #[bits(19..=20, rw)]
    write_sel: u2,
    #[bits(17..=18, rw)]
    reg_sel: u2,

    #[bit(16, rw)]
    u_we: bool,
    #[bit(15, rw)]
    v_we: bool,

    #[bits(12..=14, rw)]
    alu_op: AluOp,

    #[bit(11, rw)]
    pc_jmp: bool,
    #[bit(10, rw)]
    pc_inc: bool,

    #[bit(9, rw)]
    mar_we: bool,
    #[bit(8, rw)]
    mar_inc: bool,

    #[bit(7, rw)]
    instr_we: bool,
    #[bit(6, rw)]
    instr_step_rst: bool,
    
    #[bit(5, rw)]
    ram_hi_we: bool,
    #[bit(4, rw)]
    ram_lo_we: bool,

    #[bit(3, rw)]
    mem_hi_we: bool,
    #[bits(0..=2, rw)]
    bus_src: BusSrc,
}

#[bitenum(u3, exhaustive=true)]
#[derive(Debug)]
enum BusSrc{
    Reg,
    Alu,
    Pc, 
    Instr,
    Ram,
    Man,
    X1,
    X2
}


#[bitenum(u3, exhaustive=true)]
#[derive(Debug)]
enum AluOp{
    Add,
    Mul,
    Sub,
    And,
    Shr,
    X1,
    X2,
    X3,
}