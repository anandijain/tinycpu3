use std::{collections::HashMap, fs, path::PathBuf};

use clap::{Parser, Subcommand};
use peg;
use strum::{EnumString, FromRepr};

const N_RAM_BYTES: usize = 2_usize.pow(12);
const WORD_SIZE_IN_BYTES: usize = 2;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    Asm {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,

        #[arg(short, long)]
        asm: Option<PathBuf>
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, FromRepr)]
#[strum(ascii_case_insensitive)]
pub enum AsmOp {
    /// No op, does nothing but reset the instruction step counter
    Nop = 0,
    /// Load A, (LDA ADDR) means put the contents of RAM[addr] -> A
    Lda = 1,
    /// (ADD addr) take contents of RAM[addr] -> B, set ALU to addition, and write ALU_OUT -> A
    Add = 2,
    /// same as (ADD addr), except sets ALU to subtraction
    Sub = 3,
    /// (STA addr), store the contents of A into ram[addr]
    Sta = 4,
    /// (LDI data) store the immediate data in the lower 12 bits of the instruction into register A
    Ldi = 5,
    /// (JMP addr) set the program counter to addr
    Jmp = 6,
    /// (JMP addr) conditionally jump the PC to addr if the A register is zero
    Jz = 7,
    /// (MUL addr) multiply the contents of A with the value stored at addr, store back in A
    Mul = 8,
    ///
    Ldb = 9,
    Jeq = 10,
    And = 11,
    Shr = 12,
    Call = 13,
    Ret = 14,
    ///
    Hlt = 15,
}
impl AsmOp {
    pub fn requires_operand(self) -> bool {
        !matches!(self, Self::Nop | Self::Hlt | Self::Ret)
    }
}
peg::parser! {
    grammar asm_parser() for str {
        rule ws() = [' ' | '\t']*
        rule ws1() = [' ' | '\t']+
        rule newline() = "\r\n" / "\n" / "\r"

        rule comment() = ";" [^ '\n' | '\r']*

        rule gap() = ([' ' | '\t' | '\n' | '\r'] / comment())*
        rule line_end() = ws() comment()? (newline() / ![_])

        rule sym() -> String = op:$(['a'..='z' | 'A'..='Z' | '_']+) {op.to_string()}
        rule uint() -> usize = n:$(['0'..='9']+) {n.parse().unwrap()}

        rule sarg() -> SArg = sym:sym() {SArg::Sym(sym)} / uint:uint() {SArg::Int(uint)}

        rule instr() -> SasmExpr =
            ws() op:sym() arg:(ws1() arg:sarg() {arg})? ws() line_end() {

            SasmExpr::Inst(SInst{op: op.parse::<AsmOp>().unwrap(), arg})
        }
        rule assign() -> SasmExpr = ws() name:sym() ws1() "=" ws1() addr:uint() ws() line_end() {
            SasmExpr::Assign(name, addr)
        }
        rule label() -> SasmExpr = ws() name:sym() ":" ws() line_end() {SasmExpr::Label(name)}

        rule sasm_expr() -> SasmExpr = label() / assign() / instr()
        pub rule sasm_exprs() -> Vec<SasmExpr> = es:(e:sasm_expr() gap() {e})* {es}

    }

}

#[derive(Debug, Clone)]
enum SasmExpr {
    Inst(SInst),
    Assign(String, usize),
    Label(String),
}

/// instruction is 2 bytes: opcode<4> arg<12>
#[derive(Debug, Clone, Copy)]
struct Inst {
    op: AsmOp,
    arg: Option<usize>,
}

/// instruction is 2 bytes: opcode<4> arg<12>
#[derive(Debug, Clone)]
struct SInst {
    op: AsmOp,
    arg: Option<SArg>,
}

#[derive(Debug, Clone)]
enum SArg {
    Int(usize),
    Sym(String),
}

impl Inst {
    fn encode(self) -> u16 {
        let arg = match self.arg {
            Some(x) => x,
            None => 0,
        };
        ((self.op as usize) << 12 | arg) as u16
    }

    fn fmt_inst(self) -> String {
        if let Some(x) = self.arg {
            format!("{:?} {}\n", self.op, x)
        } else {
            format!("{:?}\n", self.op)
        }
    }
}
/// sasm -> asm -> ram
fn lower_sasm(sasm_exprs: &Vec<SasmExpr>) -> Result<Vec<Inst>, String> {
    let mut addr_table: HashMap<String, usize> = HashMap::new();
    let mut next = 0;  // addr in ram of next instruction
    let mut insts = vec![];
    for ex in sasm_exprs {
        match ex {
            SasmExpr::Inst(_) => {
                next += WORD_SIZE_IN_BYTES;
            }
            SasmExpr::Assign(name, addr) => {
                if let Some(x) = addr_table.get(name) {
                    return Err("overwriting sym disallowed for now".to_string());
                } else {
                    addr_table.insert(name.to_string(), *addr);
                }
            }
            SasmExpr::Label(name) => {
                // if label exists already in table, error.
                // else add name => next to table
                if let Some(x) = addr_table.get(name) {
                    return Err("duplicate sym name".to_string());
                } else {
                    addr_table.insert(name.to_string(), next);
                }
            }
        }
    }

    for ex in sasm_exprs {
        match ex {
            SasmExpr::Label(_) | SasmExpr::Assign(_, _) => {}
            SasmExpr::Inst(SInst { op, arg }) => {
                let concrete_arg = match arg {
                    Some(a) => match a {
                        SArg::Int(n) => Some(n),
                        SArg::Sym(s) => Some(addr_table.get(s).unwrap()),
                    },
                    None => {
                        if op.requires_operand() {
                            return Err(format!(
                                "used op {op:?} which requires arg but none provided"
                            ));
                        } else {
                            None
                        }
                    }
                };
                let inst = Inst {
                    op: *op,
                    arg: concrete_arg.copied(),
                };
                insts.push(inst);
            }
        }
    }

    Ok(insts)
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Asm { input, output, asm } => {
            let src = fs::read_to_string(&input).map_err(|e| e.to_string())?;

            let sinsts = asm_parser::sasm_exprs(&src).unwrap();
            println!("sasm_insts\n: {:?}", sinsts);

            let insts = lower_sasm(&sinsts)?;
            println!("insts\n: {:?}", insts);
            println!("formatted inst {:?}", insts[0].fmt_inst());

            if let Some(asm_path) = asm {
                let asm_src = insts.iter().flat_map(|inst| inst.fmt_inst().chars().collect::<Vec<_>>()).collect::<String>();

                fs::write(&asm_path, asm_src).map_err(|e| e.to_string())?;
                println!("wrote raw asm to {asm_path:?}");

            }

            let mut bytes = insts
                .iter()
                .flat_map(|inst| inst.encode().to_be_bytes())
                .collect::<Vec<_>>();
            bytes.resize(N_RAM_BYTES, 0);
            fs::write(&output, &bytes).map_err(|e| e.to_string())?;

            println!("wrote assembled {input:?} to {output:?}");
            
        }
    }

    Ok(())
}
