use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::Path;

static USER_TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";
static KERNEL_TARGET_PATH: &str = "target/riscv64gc-unknown-none-elf/release/";

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", USER_TARGET_PATH);
    println!("cargo:rerun-if-changed=src/");
    insert_app_data().unwrap();
    insert_kernel_symbol_elf().unwrap();
}

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .quad app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);
        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
    .align 3
app_{0}_start:
    .incbin "{2}{1}"
app_{0}_end:"#,
            idx, app, USER_TARGET_PATH
        )?;
    }
    Ok(())
}

fn insert_kernel_symbol_elf() -> Result<()> {
    let mut f = File::create("src/ksymbol.S")?;
    let symtab_path = format!("{}os.symtab", KERNEL_TARGET_PATH);
    writeln!(
        f,
        r#"
    .section .rodata
    .align 12
    .global _start_ksymbol_elf
_start_ksymbol_elf:"#
    )?;
    if Path::exists(Path::new(&symtab_path)) {
        writeln!(
            f,
            r#"
    .incbin "{}""#,
            symtab_path.as_str()
        )?;
    }
    writeln!(
        f,
        r#"
    .global _end_ksymbol_elf
_end_ksymbol_elf:"#
    )?;
    Ok(())
}
