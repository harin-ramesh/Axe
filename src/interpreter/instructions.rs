pub struct Instruction;

impl Instruction {
    // Control
    pub const HALT: u8 = 0x00;

    // Stack operations
    pub const CONST: u8 = 0x01;
    pub const POP: u8 = 0x02;
    pub const DUP: u8 = 0x03;

    // Literals
    pub const NULL: u8 = 0x04;
    pub const TRUE: u8 = 0x05;
    pub const FALSE: u8 = 0x06;

    // Arithmetic
    pub const ADD: u8 = 0x10;
    pub const SUB: u8 = 0x11;
    pub const MUL: u8 = 0x12;
    pub const DIV: u8 = 0x13;
    pub const MOD: u8 = 0x14;
    pub const NEG: u8 = 0x15;

    // Comparison
    pub const EQ: u8 = 0x20;
    pub const NEQ: u8 = 0x21;
    pub const LT: u8 = 0x22;
    pub const LTE: u8 = 0x23;
    pub const GT: u8 = 0x24;
    pub const GTE: u8 = 0x25;

    // Logical
    pub const NOT: u8 = 0x30;
    pub const AND: u8 = 0x31;
    pub const OR: u8 = 0x32;

    // Bitwise
    pub const BITAND: u8 = 0x40;
    pub const BITOR: u8 = 0x41;
    pub const BITINV: u8 = 0x42;
}
