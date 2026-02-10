pub struct Instruction;

impl Instruction {
    pub const HALT: u8  = 0x00;
    pub const LOAD: u8  = 0x01;
    pub const ADD: u8   = 0x02;
    pub const SUB: u8   = 0x03;
    pub const MUL: u8   = 0x04;
    pub const DIV: u8   = 0x05;
    pub const PRINT: u8 = 0x06;
}
