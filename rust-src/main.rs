mod format;
mod algo;
mod crypto;

use clap::{Parser, Subcommand};
use format::{ArchiveFormat, egg::EggFormat};
use std::fs;
use std::io::{self, BufReader};
use std::path::Path;

#[derive(Parser)]
#[command(name = "unegg", version = "0.5.0")]
#[command(about = "EGG/ALZ archive decompressor - Rust implementation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List files in archive
    List {
        /// Archive file path
        archive: String,
        /// Verbose level
        #[arg(short, default_value = "1")]
        verbose: u8,
    },
    /// Extract files from archive
    Extract {
        /// Archive file path
        archive: String,
        /// Destination directory
        #[arg(default_value = ".")]
        destination: String,
        /// Password for encrypted archives
        #[arg(short, long)]
        password: Option<String>,
        /// Verbose level
        #[arg(short, default_value = "1")]
        verbose: u8,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List { archive, verbose } => cmd_list(&archive, verbose),
        Commands::Extract {
            archive,
            destination,
            password,
            verbose,
        } => cmd_extract(&archive, &destination, password.as_deref(), verbose),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn open_archive(path: &str) -> io::Result<(BufReader<fs::File>, format::ArchiveInfo)> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let info = EggFormat::open(&mut reader)?;
    Ok((reader, info))
}

fn cmd_list(path: &str, verbose: u8) -> io::Result<()> {
    let (mut reader, info) = open_archive(path)?;

    println!();
    println!("unegg v0.5.0 (Rust)");
    println!();

    if verbose >= 1 {
        println!("Global informations");
        println!("[");
        println!("\tFormat : EGG");
        println!("\tVersion : {}.{}", info.version_major, info.version_minor);
        println!("\tSolid : {}", if info.is_solid { "yes" } else { "no" });
        println!("\tSpanned : {}", if info.is_spanned { "yes" } else { "no" });
        println!("\tFile Count : {}", info.files.len());
        if let Some(ref comment) = info.comment {
            println!("\tGlobal Comment : {comment}");
        } else {
            println!("\tGlobal Comment : none");
        }
        println!("]");
        println!();
    }

    let total_cols = 120;
    let separator = "-".repeat(total_cols);

    if verbose < 2 {
        println!("{separator}");
        println!(
            " {:>11} | {:>11} | {:>8} | {:>11} | {:>11} | {:>11} | {}",
            "Packed", "Unpacked", "Blocks", "Encrypted", "Algorithm", "CRC", "Name"
        );
        println!("{separator}");
    }

    for file in &info.files {
        let algo_name = file
            .blocks
            .first()
            .map(|b| b.method.name())
            .unwrap_or("UNKNOWN");

        let total_packed: u32 = file.blocks.iter().map(|b| b.comp_size).sum();
        let block_count = file.blocks.len();

        if verbose < 2 {
            println!(
                " {:>11} | {:>11} | {:>8} | {:>11} | {:>11} | {:>11} | {}",
                total_packed,
                file.uncomp_size,
                block_count,
                if file.is_encrypted { "yes" } else { "no" },
                algo_name,
                file.blocks.first().map(|b| format!("0x{:08x}", b.crc)).unwrap_or_default(),
                file.name,
            );
        } else {
            println!("{}", file.name);
            println!("[");
            println!("\t{:>11} : {}", "Packed", total_packed);
            println!("\t{:>11} : {}", "Unpacked", file.uncomp_size);
            println!("\t{:>11} : {}", "Blocks", block_count);
            println!("\t{:>11} : {}", "Encrypted", if file.is_encrypted { "yes" } else { "no" });
            println!("\t{:>11} : {}", "Algorithm", algo_name);
            if let Some(b) = file.blocks.first() {
                println!("\t{:>11} : 0x{:08x}", "CRC", b.crc);
            }
            println!("]");
        }
    }

    println!("{separator}");
    Ok(())
}

fn cmd_extract(path: &str, dest: &str, password: Option<&str>, verbose: u8) -> io::Result<()> {
    let (mut reader, info) = open_archive(path)?;

    println!();
    println!("unegg v0.5.0 (Rust)");
    println!();

    let dest_path = Path::new(dest);
    let mut total_files = 0u32;

    for file in &info.files {
        if verbose >= 1 {
            println!("Extracting: {}", file.name);
        }

        let output_path = dest_path.join(&file.name);

        // Create parent directories
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Check if file has blocks (data)
        if file.blocks.is_empty() {
            if verbose >= 2 {
                println!("  (empty file)");
            }
            fs::write(&output_path, &[])?;
        } else {
            let data = EggFormat::extract_file(&mut reader, file, password)?;
            fs::write(&output_path, &data)?;

            if verbose >= 2 {
                println!("  {} bytes", data.len());
            }
        }

        total_files += 1;
    }

    if verbose >= 1 {
        println!();
        println!("Total: {total_files} File(s)");
        println!();
    }

    Ok(())
}
