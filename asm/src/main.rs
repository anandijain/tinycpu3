use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use peg;
use strum::{EnumString, FromRepr};

mod control;

const N_RAM_BYTES: usize = 2_usize.pow(12);

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
    },
    Control {
        // #[arg(short, long, value_parser = parse_uint)]
        // input: usize,
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        name: String,
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

peg::parser! {
    grammar asm_parser() for str {
        rule ws() = [' ' | '\t']*
        rule ws1() = [' ' | '\t']+



        rule op() -> String = op:$(['a'..='z' | 'A'..='Z']+) {op.to_string()}
        rule arg() -> usize = n:$(['0'..='9']+) {n.parse().unwrap()}

        rule instr() -> Inst = ws() op:op() arg:(ws1() n:arg() {n})? ws() {Inst{op: op.parse::<AsmOp>().unwrap(), arg}}
        pub rule instrs() -> Vec<Inst> = insts:(instr() ** "\n") {insts}
    }

}

/// instruction is 2 bytes: opcode<4> arg<12>
#[derive(Debug, Clone, Copy)]
struct Inst {
    op: AsmOp,
    arg: Option<usize>,
}

impl Inst {
    fn encode(self) -> u16 {
        let arg = match self.arg {
            Some(x) => x,
            None => 0,
        };
        ((self.op as usize) << 12 | arg) as u16
    }
}
use std::num::ParseIntError;

fn parse_uint(s: &str) -> Result<usize, ParseIntError> {
    let (digits, radix) = if let Some(rest) = s.strip_prefix("0x") {
        (rest, 16)
    } else if let Some(rest) = s.strip_prefix("0b") {
        (rest, 2)
    } else {
        (s, 10)
    };

    usize::from_str_radix(digits, radix)
}

fn print_signal(path: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut wave = wellen::simple::read(path)?;

    let hierarchy = wave.hierarchy();

    // Find the signal by its full hierarchical name.
    let var = hierarchy
        .all_vars()
        .find(|&v| hierarchy[v].full_name(hierarchy) == name)
        .ok_or("signal not found")?;

    let signal_ref = hierarchy[var].signal_ref();

    wave.load_signals(&[signal_ref]);
    let signal = wave.get_signal(signal_ref).unwrap();

    for (time_index, value) in signal.iter_changes() {
        let time = wave.time_table()[time_index as usize];
        println!("t={time}: {value}");
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Asm { input, output } => {
            let src = fs::read_to_string(input).map_err(|e| e.to_string())?;

            let insts = asm_parser::instrs(&src).unwrap();
            let mut bytes = insts
                .iter()
                .flat_map(|inst| inst.encode().to_be_bytes())
                .collect::<Vec<_>>();
            bytes.resize(N_RAM_BYTES, 0);
            fs::write(&output, &bytes).map_err(|e| e.to_string())?;

            println!("{:?}", insts);
        }
        Commands::Control { input, name } => {
            let full_name = format!("tinycpu3::main.{name}");
            let mut wave = wellen::simple::read(input).unwrap();

            let hierarchy = wave.hierarchy();

            // Find the signal by its full hierarchical name.
            let var = hierarchy
                .all_vars()
                .find(|&v| hierarchy[v].full_name(hierarchy) == full_name)
                .ok_or("signal not found")?;

            let signal_ref = hierarchy[var].signal_ref();

            wave.load_signals(&[signal_ref]);
            let signal = wave.get_signal(signal_ref).unwrap();

            for (time_index, value) in signal.iter_changes() {
                match u64::try_from(value) {
                    Ok(raw) => {
                        let raw = u32::try_from(raw).unwrap();
                        let bits = arbitrary_int::u22::try_new(raw).map_err(|e| e.to_string())?;

                        let control = control::Control::new_with_raw_value(bits);
                        println!("{control:#?}");
                    }
                    Err(_) => {
                        println!("{value}"); // e.g. bits containing x or z
                    }
                }
            }
            // let bits = arbitrary_int::u22::try_new(input as u32).unwrap();
            // let c = control::Control::new_with_raw_value(bits);
            // println!("{:#?}", c);
        }
    }
    // println!("{:?}", "0x55".parse::<usize>());

    Ok(())
}
