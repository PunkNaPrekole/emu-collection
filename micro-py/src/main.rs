use std::fs;
use clap::{Parser, Subcommand};

mod error;
mod ir;
mod parser;
mod backends;
pub mod span;

use backends::BackendType;

#[derive(Parser)]
#[command(name = "micro-py")]
#[command(about = "Python-like compiler for retro architectures", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Компилирует код под выбранную архитектуру
    Compile {
        /// Исходник
        input: String,
        
        /// Нужная архитектура
        #[arg(short, long, default_value = "chip8")]
        target: String,
        
        /// Скомпилированный файл
        #[arg(short, long)]
        output: Option<String>,
        
        /// Показать ast
        #[arg(long)]
        show_ast: bool,
    },
    
    /// Распарсить и показать ast без компиляции
    Parse {
        /// Исходник
        input: String,
    },
    
    /// Список поддерживаемых архитектур
    Targets,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, target, output, show_ast } => {
            println!("Compiling {} for {}...", input, target);
            
            let source = fs::read_to_string(&input)?;
            let program = parser::parse(&source)?;

            if show_ast {
                println!("=== AST ===");
                println!("{:#?}", program);
            }
            
            match BackendType::all().iter().find(|b| b.name() == target) {
                Some(backend_type) => {
                    let mut backend = backend_type.create();
                    let machine_code = backend.compile(&program)?;

                    let output_path = match output {
                        Some(path) => path,
                        None => {
                            // Автоматическое имя: input.ch8 или input.bin
                            let base_name = input.trim_end_matches(".py");
                            match target.as_str() {
                                "chip8" => format!("{}.ch8", base_name),
                                _ => format!("{}.bin", base_name),
                            }
                            
                        }
                    };
                    
                    // Сохраняем в файл
                    fs::write(&output_path, &machine_code)?;
                    println!("Compiled to: {}", output_path);
                    println!("Code size: {} bytes", machine_code.len());
                    
                    // Показываем дизассемблированный код
                    println!("Disassembly:");
                    for (i, chunk) in machine_code.chunks(2).enumerate() {
                        if chunk.len() == 2 {
                            let instruction = ((chunk[0] as u16) << 8) | (chunk[1] as u16);
                            println!("  0x{:03X}: {:04X}", 0x200 + i * 2, instruction);
                        }
                    }
                }
                _ => {
                    eprintln!("Unknown target: {}", target);
                }
            }
        }
        Commands::Parse { input } => {
            println!("Parsing {}...", input);
            
            let source = fs::read_to_string(&input)?;
            
            println!("=== SOURCE ===");
            println!("{}", source);
            println!("=== TOKENS ===");
            
            let tokens = parser::lexer::tokenize(&source)?;
            for token in &tokens {
                println!("{:?}", token);
            }
            
            println!("=== AST ===");
            match parser::parse(&source) {
                Ok(program) => {
                    println!("{:#?}", program);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
        Commands::Targets => {
            println!("Supported targets:");
            for backend in BackendType::all() {
                println!("  {:8} - {}", backend.name(), backend.description());
            }
        }
    }
    
    Ok(())
}
