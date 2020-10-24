use std::{
    env,
    fs::File,
    io::{self, prelude::*},
    path::PathBuf,
};

fn main() -> Result<(), Error> {
    let target = Target::read();

    copy_memory_config(target)?;

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Make `memory.x` available to dependent crates
fn copy_memory_config(target: Target) -> Result<(), Error> {
    let memory_x = match target.sub_family {
        SubFamily::Stm32f405 => include_bytes!("memory_512_128.x").as_ref(),
        SubFamily::Stm32f446 => include_bytes!("memory_512_128.x").as_ref(),
        SubFamily::Stm32f767 => include_bytes!("memory_2048_368.x").as_ref(),
    };

    let out_dir = env::var("OUT_DIR")?;
    let out_dir = PathBuf::from(out_dir);

    File::create(out_dir.join("memory.x"))?.write_all(memory_x)?;

    // Tell Cargo where to find the file.
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=memory_2048_368.x");
    println!("cargo:rerun-if-changed=memory_512_128.x");

    Ok(())
}

#[derive(Clone, Copy)]
struct Target {
    sub_family: SubFamily,
}

impl Target {
    fn read() -> Self {
        let sub_family = SubFamily::read();

        Self { sub_family }
    }
}

#[derive(Clone, Copy)]
enum SubFamily {
    Stm32f405,
    Stm32f446,
    Stm32f767,
}

impl SubFamily {
    fn read() -> Self {
        if cfg!(feature = "nucleof446re") {
            SubFamily::Stm32f446
        } else if cfg!(feature = "nucleo767zi") {
            SubFamily::Stm32f767
        } else if cfg!(feature = "production") {
            SubFamily::Stm32f405
        } else {
            error("You must select a target.
If you added Stm32f7xx HAL as a dependency to your crate, you can select a target by enabling the respective feature in `Cargo.toml`.
If you're running an example from the repository, select a target by passing the desired target as a command-line argument, for example `--features=stm32f746`.
Please refer to the documentation for more details."
                )
        }
    }
}

#[derive(Debug)]
enum Error {
    Env(env::VarError),
    Io(io::Error),
}

impl From<env::VarError> for Error {
    fn from(error: env::VarError) -> Self {
        Self::Env(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn error(message: &str) -> ! {
    panic!("\n\n\n{}\n\n\n", message);
}
