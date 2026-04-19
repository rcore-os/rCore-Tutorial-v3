use riscv_decode::{self, Instruction};
pub enum Register {
    Zero,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    Fp,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
}

impl From<u32> for Register {
    fn from(value: u32) -> Self {
        match value as u8 {
            0 => Register::Zero,
            1 => Register::Ra,
            2 => Register::Sp,
            3 => Register::Gp,
            4 => Register::Tp,
            5 => Register::T0,
            6 => Register::T1,
            7 => Register::T2,
            8 => Register::Fp,
            9 => Register::S1,
            10 => Register::A0,
            11 => Register::A1,
            12 => Register::A2,
            13 => Register::A3,
            14 => Register::A4,
            15 => Register::A5,
            16 => Register::A6,
            17 => Register::A7,
            18 => Register::S2,
            19 => Register::S3,
            20 => Register::S4,
            21 => Register::S5,
            22 => Register::S6,
            23 => Register::S7,
            24 => Register::S8,
            25 => Register::S9,
            26 => Register::S10,
            27 => Register::S11,
            28 => Register::T3,
            29 => Register::T4,
            30 => Register::T5,
            31 => Register::T6,
            _ => panic!("Wrong register in riscv!"),
        }
    }
}

impl Register {
    pub fn to_str(&self) -> &'static str {
        match self {
            Register::Zero => "zero",
            Register::Ra => "ra",
            Register::Sp => "sp",
            Register::Gp => "gp",
            Register::Tp => "tp",
            Register::T0 => "t0",
            Register::T1 => "t1",
            Register::T2 => "t2",
            Register::Fp => "fp",
            Register::S1 => "s1",
            Register::A0 => "a0",
            Register::A1 => "a1",
            Register::A2 => "a2",
            Register::A3 => "a3",
            Register::A4 => "a4",
            Register::A5 => "a5",
            Register::A6 => "a6",
            Register::A7 => "a7",
            Register::S2 => "s2",
            Register::S3 => "s3",
            Register::S4 => "s4",
            Register::S5 => "s5",
            Register::S6 => "s6",
            Register::S7 => "s7",
            Register::S8 => "s8",
            Register::S9 => "s9",
            Register::S10 => "s10",
            Register::S11 => "s11",
            Register::T3 => "t3",
            Register::T4 => "t4",
            Register::T5 => "t5",
            Register::T6 => "t6",
        }
    }
}

pub fn print_load_store(ins: &Instruction) {
    match *ins {
        Instruction::Lb(it) => {
            println!(
                "lb {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Lbu(it) => {
            println!(
                "lbu {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Lh(it) => {
            println!(
                "lh {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Lhu(it) => {
            println!(
                "lhu {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Lw(it) => {
            println!(
                "lw {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Lwu(it) => {
            println!(
                "lwu {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Ld(it) => {
            println!(
                "ld {}, {}({})",
                Register::from(it.rd()).to_str(),
                it.imm(),
                Register::from(it.rs1()).to_str(),
            );
        }
        Instruction::Sb(st) => {
            println!(
                "sb {}, {}({})",
                Register::from(st.rs2()).to_str(),
                st.imm(),
                Register::from(st.rs1()).to_str(),
            );
        }
        Instruction::Sh(st) => {
            println!(
                "sh {}, {}({})",
                Register::from(st.rs2()).to_str(),
                st.imm(),
                Register::from(st.rs1()).to_str()
            );
        }
        Instruction::Sw(st) => {
            println!(
                "sw {}, {}({})",
                Register::from(st.rs2()).to_str(),
                st.imm(),
                Register::from(st.rs1()).to_str()
            );
        }
        Instruction::Sd(st) => {
            println!(
                "sd {}, {}({})",
                Register::from(st.rs2()).to_str(),
                st.imm(),
                Register::from(st.rs1()).to_str()
            );
        }
        _ => panic!("Not load or store instruction!"),
    }
}
