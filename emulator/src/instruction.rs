//! Representation of a Chip-8 CPU instruction

/// An address in memory
type Address = u16;
/// A value in memory
type Number = u8;
/// An ID of a register
type Register = u8;

#[cfg_attr(feature = "clippy", allow(doc_markdown))]
/// An instruction
/// For information about the instruction set, see:
/// https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
pub enum Instruction {
    // Flow
    /// Return from subroutine
    Return,
    /// Goto the address
    Goto(Address),
    /// Call the subroutine at the address
    Call(Address),
    /// Goto the address + V0
    OffsetGoto(Address),

    // Const
    /// Sets VX to N
    SetConst(Register, Number),
    /// Adds N to VX
    AddConst(Register, Number),

    // Assign
    /// Sets VX to VY
    Move(Register, Register),

    // BitOp
    /// Sets VX to VX | VY
    BitOr(Register, Register),
    /// Sets VX to VX & VY
    BitAnd(Register, Register),
    /// Sets VX to VX ^ VY
    BitXor(Register, Register),
    /// Shifts VX to the right by one
    Shr(Register),
    /// Shifts VX to the left by one
    Shl(Register),

    // Math
    /// Adds VY to VX
    Add(Register, Register),
    /// Subtracts VY from VX
    Sub(Register, Register),
    /// Sets VX to VY - VX
    InverseSub(Register, Register),

    // Rand
    /// Sets VX to rand() & N
    Rand(Register, Number),

    // BCD
    /// Writes the BCD representation of VX to memory at addresses I, I + 1, and I + 2
    BCD(Register),

    // Cond
    /// Skips the next instruction if VX == N
    SkipEqConst(Register, Number),
    /// Skips the next instruction if VX != N
    SkipNeqConst(Register, Number),
    /// Skips the next instruction if VX == VY
    SkipEq(Register, Register),
    /// Skips the next instruction if VX != VY
    SkipNeq(Register, Register),

    // MEM
    /// Writes registers V0 through VX to memory starting at address I
    RegDump(Register),
    /// Loads bytes in memory starting at address I into registers V0 through VX
    RegLoad(Register),
    /// Sets I to N
    SetIndex(Address),
    /// Adds N to I
    AddIndex(Register),
    /// Sets I to the address of the sprite of the character stored in VX
    SetIndexChar(Register),

    // Timer
    /// Sets VX to the delay timer
    GetDelay(Register),
    /// Sets the delay timer to VX
    SetDelay(Register),

    // KeyOp
    /// Blocks until a key is pressed, then stores it in VX
    WaitKey(Register),
    /// Skips the next instruction if the key in VX is pressed
    SkipKey(Register),
    /// Skips the next instruction if the key in VX is not pressed
    SkipNotKey(Register),

    // Sound
    /// Sets the sound timer to VX
    SetSound(Register),

    // Disp
    /// Loads a sprite that is 8 pixels wide and N pixels tall from memory starting at address I,
    /// and draws it at coordinate (VX, VY)
    Draw(Register, Register, Number),
    /// Clears the screen
    ClearScreen,
}
