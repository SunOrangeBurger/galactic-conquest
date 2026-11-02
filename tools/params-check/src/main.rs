use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let path = env::args().nth(1).unwrap_or_else(|| "params.toml".into());
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("{path} is not UTF-8: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = toml::from_str::<toml::Table>(text) {
        eprintln!("invalid TOML in {path}: {error}");
        return ExitCode::FAILURE;
    }

    // FNV-1a over the exact file bytes gives a stable configuration identity
    // without relying on HashMap iteration or platform-specific hashers.
    let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    });
    println!("{} fnv1a64:{hash:016x}", path);
    ExitCode::SUCCESS
}
