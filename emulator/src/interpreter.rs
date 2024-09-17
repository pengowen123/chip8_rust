//! Interpretation of opcodes

use errors::*;

use instruction::Instruction;

/// A helper macro to shorten the creation of instructions
///
/// # Examples
///
/// ```rust
/// assert_eq!(
///     Instruction::SetConst(nibble(0xABCD, 1), nibbles(0xABCD, 2, 3)),
///     instruction!(0xABCD, SetConst(1, [2, 3]))
/// );
/// ```
macro_rules! instruction {
    // Helper; matches a nibble range
    (FIELD, $opcode:expr, [$start:expr, $end:expr]) => {{
        nibbles($opcode, $start, $end)
    }};
    // Helper; matches a nibble index
    (FIELD, $opcode:expr, $index:tt) => {{
        nibble($opcode, $index)
    }};
    // Actual macro
    ($opcode:expr, $variant:ident($($field:tt),+)) => {{
        $variant($(instruction!(FIELD, $opcode, $field)),*)
    }};
}

/// Returns an instruction, interpreted from an opcode
/// Returns an error if the opcode is not a valid instruction

// Prevent rustfmt from ruining the formatting of the match arms
#[cfg_attr(rustfmt, rustfmt_skip)]
pub fn interpret_instruction(opcode: u16) -> Result<Instruction> {
    use instruction::Instruction::*;

    // To reduce boilerplate, the `instruction` macro is used to create instructions
    // See the docs for the macro to understand what this code is doing
    let instruction = match (nibble(opcode, 0),
                             nibble(opcode, 1),
                             nibble(opcode, 2),
                             nibble(opcode, 3)) {

        // Flow
        (0x0, 0x0, 0xE, 0xE) =>                      Return,
        (0x1, ..)            =>                      Goto(opcode & 0x0FFF),
        (0x2, ..)            =>                      Call(opcode & 0x0FFF),
        (0xB, ..)            =>                      OffsetGoto(opcode & 0xFFF),

        // Const
        (0x6, ..)            => instruction!(opcode, SetConst(1, [2, 3])),
        (0x7, ..)            => instruction!(opcode, AddConst(1, [2, 3])),

        // Assign
        (0x8, .., 0x0)       => instruction!(opcode, Move(1, 2)),

        // BitOp
        (0x8, .., 0x1)       => instruction!(opcode, BitOr(1, 2)),
        (0x8, .., 0x2)       => instruction!(opcode, BitAnd(1, 2)),
        (0x8, .., 0x3)       => instruction!(opcode, BitXor(1, 2)),
        (0x8, .., 0x6)       => instruction!(opcode, Shr(1)),
        (0x8, .., 0xE)       => instruction!(opcode, Shl(1)),

        // Math
        (0x8, .., 0x4)       => instruction!(opcode, Add(1, 2)),
        (0x8, .., 0x5)       => instruction!(opcode, Sub(1, 2)),
        (0x8, .., 0x7)       => instruction!(opcode, InverseSub(1, 2)),

        // Rand
        (0xC, ..)            => instruction!(opcode, Rand(1, [2, 3])),

        // BCD
        (0xF, _, 0x3, 0x3)   => instruction!(opcode, BCD(1)),

        // Cond
        (0x3, ..)            => instruction!(opcode, SkipEqConst(1, [2, 3])),
        (0x4, ..)            => instruction!(opcode, SkipNeqConst(1, [2, 3])),
        (0x5, .., 0x0)       => instruction!(opcode, SkipEq(1, 2)),
        (0x9, .., 0x0)       => instruction!(opcode, SkipNeq(1, 2)),

        // MEM
        (0xF, _, 0x5, 0x5)   => instruction!(opcode, RegDump(1)),
        (0xF, _, 0x6, 0x5)   => instruction!(opcode, RegLoad(1)),
        (0xA, ..)            =>                      SetIndex(opcode & 0x0FFF),
        (0xF, _, 0x1, 0xE)   => instruction!(opcode, AddIndex(1)),
        (0xF, _, 0x2, 0x9)   => instruction!(opcode, SetIndexChar(1)),

        // Timer
        (0xF, _, 0x0, 0x7)   => instruction!(opcode, GetDelay(1)),
        (0xF, _, 0x1, 0x5)   => instruction!(opcode, SetDelay(1)),

        // KeyOp
        (0xF, _, 0x0, 0xA)   => instruction!(opcode, WaitKey(1)),
        (0xE, _, 0x9, 0xE)   => instruction!(opcode, SkipKey(1)),
        (0xE, _, 0xA, 0x1)   => instruction!(opcode, SkipNotKey(1)),

        // Sound
        (0xF, _, 0x1, 0x8)   => instruction!(opcode, SetSound(1)),


        (0xD, ..)            => instruction!(opcode, Draw(1, 2, 3)),
        (0x0, 0x0, 0xE, 0x0) =>                      ClearScreen,

        // Invalid instruction
        _ => bail!(ErrorKind::InvalidOpcode(format!("0x{:04X}", opcode))),
    };

    Ok(instruction)
}

/// A helper function to select nibbles from a number and convert them to bytes
/// The range is inclusive
///
/// # Examples
///
/// ```rust,no_run
/// # // This is a private function, so here is a dummy to make the test pass
/// # let nibbles = |a: u16, b: u8, c: u8| { a };
/// assert_eq!(0xBC, nibbles(0xABCD, 1, 2))
/// ```
fn nibbles(num: u16, start: u8, end: u8) -> u8 {
    let end = end + 1;
    let range = end - start;
    let initial = 0xFFFF - (16u16.pow(4 - range as u32) - 1);
    let masked = num & (initial >> (start * 4));
    (masked >> (4 * (4 - end))) as u8
}

/// Equivalent to `nibbles(num, index, index)`
fn nibble(num: u16, index: u8) -> u8 {
    nibbles(num, index, index)
}

#[cfg(test)]
mod tests {
    use super::{nibble, nibbles};

    #[test]
    fn test_nibble() {
        assert_eq!(0xB, nibble(0xABCD, 1));
    }

    #[test]
    fn test_nibbles() {
        assert_eq!(0xBC, nibbles(0xABCD, 1, 2));
    }
}
