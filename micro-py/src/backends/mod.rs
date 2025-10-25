pub mod chip8;

use crate::ir::ast;
use crate::error::CompileError;

pub trait Backend {
    fn compile(&mut self, program: &ast::Program) -> Result<Vec<u8>, CompileError>;
}

#[derive(Debug, Clone, Copy)]
pub enum BackendType {
    Chip8,
    // I8080,
    // Z80, 
    // 6502,
}

impl BackendType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Chip8,
            // Self::I8080,
            // Self::Z80,
            // Self::6502,
        ]
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::Chip8 => "chip8",
            // Self::I8080 => "i8080",
            // Self::Z80 => "z80", 
            // Self::6502 => "6502",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            Self::Chip8 => "CHIP-8 virtual machine",
            // Self::I8080 => "Intel 8080 CPU",
            // Self::Z80 => "Zilog Z80 CPU",
            // Self::6502 => "MOS 6502 CPU",
        }
    }
    
    pub fn create(&self) -> Box<dyn Backend> {
        match self {
            Self::Chip8 => Box::new(chip8::Chip8Backend::new()),
            // Self::I8080 => Box::new(I8080Backend::new()),
            // Self::Z80 => Box::new(Z80Backend::new()),
            // Self::6502 => Box::new(MOS6502Backend::new()),
        }
    }
}