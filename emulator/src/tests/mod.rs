//! Tests for the Chip-8 emulator
//! To read the test programs, see the opcode table at:
//! https://en.wikipedia.org/wiki/CHIP-8#Opcode_table
//!
//! Most tests rely on other instructions working correctly
//! Because of this, if one instruction stops working, many tests may fail

#[macro_use]
mod utils;

use self::utils::*;
use Chip8;
use config::Log;
use errors::*;

/// A version of `chip8::run` that runs a program, then returns the emulator and I/O state for
/// testing
/// Timers are updated once per cycle rather than at 60 hz
/// `Some(cycles)` can be passes to override the default calculation of cycles to run
fn run_program<I>(program: &[u8],
                  keypresses: Option<Vec<Keypress>>,
                  cycles: Option<usize>)
                  -> (Chip8, I)
    where I: TestIO + ::Chip8IO
{
    let mut chip8 = Chip8::new(program, Log::Disabled).unwrap();
    let mut io = I::new(keypresses.unwrap_or(Vec::new()));

    // Two bytes is one instruction, so only run half as many cycles as there are bytes
    // NOTE: If a test program relies on control flow, pass `Some(cycles)` to control how many
    //       cycles are run
    for _ in 0..cycles.unwrap_or(program.len() / 2) {
        // Simulate key presses
        io.simulate_keypresses();
        // Run a CPU cycle
        chip8.cycle(&mut io).unwrap();
        // Countdown timers
        // Not simulated at the correct speed, but still useful to test whether they work
        chip8.update_timers(&mut io);
    }

    (chip8, io)
}

fn run_program_default(program: &[u8]) -> Chip8 {
    run_program::<Io>(program, None, None).0
}

/// Tests that the emulator won't run programs that are too large
#[test]
fn program_too_large() {
    let program = [0; ::MEMORY];
    let chip8 = Chip8::new(&program, Log::Disabled);

    match chip8 {
        Err(Error(ErrorKind::ProgramTooLarge(..), _)) => {}
        Err(e) => panic!("Wrong error: {}", e),
        Ok(_) => panic!("Expected error"),
    }
}

/// Tests instruction SetConst
#[test]
fn set_const() {
    let program = program!(0x6040);

    let chip8 = run_program_default(&program);

    assert_eq!(0x40, chip8.registers.get(0));
}

/// Tests instruction AddConst
#[test]
fn add_const() {
    let program = program!(0x7040);

    let chip8 = run_program_default(&program);

    assert_eq!(0x40, chip8.registers.get(0));
}

/// Tests that AddConst wraps
#[test]
fn add_const_wrap() {
    let program = program!(0x60FF, 0x7041);

    let chip8 = run_program_default(&program);

    assert_eq!(0x40, chip8.registers.get(0));
}

/// Tests instruction Move
#[test]
fn move_() {
    let program = program!(0x6040, 0x8100);

    let chip8 = run_program_default(&program);

    assert_eq!(0x40, chip8.registers.get(1));
}

/// Tests instruction BitOr
#[test]
fn bitor() {
    let program = program!(0x60F0, 0x610F, 0x8011);

    let chip8 = run_program_default(&program);

    assert_eq!(0xFF, chip8.registers.get(0));
}

/// Tests instruction BitAnd
#[test]
fn bitand() {
    let program = program!(0x60FA, 0x61AF, 0x8012);

    let chip8 = run_program_default(&program);

    assert_eq!(0xAA, chip8.registers.get(0));
}

/// Tests instruction BitXor
#[test]
fn bitxor() {
    let program = program!(0x60FA, 0x61AF, 0x8013);

    let chip8 = run_program_default(&program);

    assert_eq!(0x55, chip8.registers.get(0));
}

/// Tests instruction Shr
#[test]
fn shr() {
    let program = program!(0x60F0, 0x8006);

    let chip8 = run_program_default(&program);

    assert_eq!(0x78, chip8.registers.get(0));
}

/// Tests that Shr sets VF to the least significant bit (LSB) of VX before the shift
/// In this case, that value should be 0
#[test]
fn shr_lsb_0() {
    let program = program!(0x60F0, 0x8006);

    let chip8 = run_program_default(&program);

    assert_eq!(0x00, chip8.registers.get(0xF));
}

/// Tests that Shr sets VF to the least significant bit (LSB) of VX before the shift
/// In this case, that value should be 1
#[test]
fn shr_lsb_1() {
    let program = program!(0x60F1, 0x8006);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(0xF));
}

/// Tests instruction Shl
#[test]
fn shl() {
    let program = program!(0x608F, 0x800E);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1E, chip8.registers.get(0));
}

/// Tests that Shl sets VF to the most significant bit (MSB) of VX before the shift
/// In this case, that value should be 0
#[test]
fn shl_msb_0() {
    let program = program!(0x600F, 0x800E);

    let chip8 = run_program_default(&program);

    assert_eq!(0x0, chip8.registers.get(0xF));
}

/// Tests that Shl sets VF to the most significant bit (MSB) of VX before the shift
/// In this case, that value should be 1
#[test]
fn shl_msb_1() {
    let program = program!(0x608F, 0x800E);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(0xF));
}

/// Tests instruction Add
#[test]
fn add() {
    let program = program!(0x6010, 0x6120, 0x8014);

    let chip8 = run_program_default(&program);

    assert_eq!(0x30, chip8.registers.get(0));
}

/// Tests that Add wraps
#[test]
fn add_wrap() {
    let program = program!(0x60FF, 0x6141, 0x8014);

    let chip8 = run_program_default(&program);

    assert_eq!(0x40, chip8.registers.get(0));
}

/// Tests that Add sets VF to zero when a carry doesn't happen
#[test]
fn add_carry_0() {
    let program = program!(0x6010, 0x6120, 0x6F01, 0x8014);

    let chip8 = run_program_default(&program);

    assert_eq!(0x00, chip8.registers.get(0xF));
}

/// Tests that Add sets VF to one when a carry happens
#[test]
fn add_carry_1() {
    let program = program!(0x60FF, 0x6101, 0x6F01, 0x8014);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(0xF));
}

/// Tests instruction Sub
#[test]
fn sub() {
    let program = program!(0x6030, 0x6110, 0x8015);

    let chip8 = run_program_default(&program);

    assert_eq!(0x20, chip8.registers.get(0));
}

/// Tests that Sub wraps
#[test]
fn sub_wrap() {
    let program = program!(0x6000, 0x6110, 0x8015);

    let chip8 = run_program_default(&program);

    assert_eq!(0xF0, chip8.registers.get(0));
}

/// Tests that Sub sets VF to zero when a borrow doesn't happen
#[test]
fn sub_borrow_0() {
    let program = program!(0x6001, 0x6101, 0x6F01, 0x8015);

    let chip8 = run_program_default(&program);

    assert_eq!(0x00, chip8.registers.get(0xF));
}

/// Tests that Sub sets VF to one when a borrow happens
#[test]
fn sub_borrow_1() {
    let program = program!(0x6000, 0x6101, 0x6F01, 0x8015);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(0xF));
}

/// Tests instruction InverseSub
#[test]
fn inverse_sub() {
    let program = program!(0x6010, 0x6130, 0x8017);

    let chip8 = run_program_default(&program);

    assert_eq!(0x20, chip8.registers.get(0));
}

/// Tests that InverseSub wraps
#[test]
fn inverse_sub_wrap() {
    let program = program!(0x6010, 0x6100, 0x8017);

    let chip8 = run_program_default(&program);

    assert_eq!(0xF0, chip8.registers.get(0));
}

/// Tests that InverseSub sets VF to zero when a borrow doesn't happen
#[test]
fn inverse_sub_borrow_0() {
    let program = program!(0x6001, 0x6101, 0x6F01, 0x8017);

    let chip8 = run_program_default(&program);

    assert_eq!(0x00, chip8.registers.get(0xF));
}

/// Tests that InverseSub sets VF to one when a borrow happens
#[test]
fn inverse_sub_borrow_1() {
    let program = program!(0x6001, 0x6100, 0x6F01, 0x8017);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(0xF));
}

/// Tests instruction BCD
#[test]
fn bcd() {
    let program = program!(0x60FF, 0xF033);

    let chip8 = run_program_default(&program);

    let bcd = chip8.memory[0] * 100 + chip8.memory[1] * 10 + chip8.memory[2];
    assert_eq!(bcd, 0xFF);
}

/// Tests that BCD writes to the correct address
#[test]
fn bcd_address() {
    let program = program!(0x60FF, 0xA100, 0xF033);

    let chip8 = run_program_default(&program);

    let bcd = chip8.memory[256] * 100 + chip8.memory[257] * 10 + chip8.memory[258];
    assert_eq!(bcd, 0xFF);
}

/// Tests instruction SkipEqConst when the skip should happen
#[test]
fn skip_eq_const_0() {
    // NOTE: The 0x6100 at the end sets V1 to 0
    //       If the instruction is skipped, this instruction is run so V1 is 0
    //       If the instruction is not skipped, 0x61FF will run instead, setting V1 to 255, so the
    //       test fails
    //       This method of testing is also used in the rest of the tests for skip instructions
    let program = program!(0x6001, 0x3001, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0x00, chip8.registers.get(1));
}

/// Tests instruction SkipEqConst when the skip should not happen
#[test]
fn skip_eq_const_1() {
    let program = program!(0x6000, 0x3001, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0xFF, chip8.registers.get(1));
}

/// Tests instruction SkipNeqConst when the skip should not happen
#[test]
fn skip_neq_const_0() {
    let program = program!(0x6001, 0x4001, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0xFF, chip8.registers.get(1));
}

/// Tests instruction SkipNeqConst when the skip should happen
#[test]
fn skip_neq_const_1() {
    let program = program!(0x6000, 0x4001, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0x00, chip8.registers.get(1));
}

/// Tests instruction SkipEq when the skip should happen
#[test]
fn skip_eq_0() {
    let program = program!(0x6000, 0x5010, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0x00, chip8.registers.get(1));
}

/// Tests instruction SkipEq when the skip should not happen
#[test]
fn skip_eq_1() {
    let program = program!(0x6001, 0x5010, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0xFF, chip8.registers.get(1));
}

/// Tests instruction SkipNeq when the skip should not happen
#[test]
fn skip_neq_0() {
    let program = program!(0x6000, 0x9010, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0xFF, chip8.registers.get(1));
}

/// Tests instruction SkipNeq when the skip should happen
#[test]
fn skip_neq_1() {
    let program = program!(0x6001, 0x9010, 0x61FF, 0x6100);

    let (chip8, _) = run_program::<Io>(&program, None, Some(3));

    assert_eq!(0x00, chip8.registers.get(1));
}

/// Tests instruction RegDump
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn reg_dump() {
    // Fill the registers with the numbers 0 to 15, then dump them at address 0
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0x6505, 0x6606, 0x6707, 0x6808,
                           0x6909, 0x6A0A, 0x6B0B, 0x6C0C, 0x6D0D, 0x6E0E, 0x6F0F, 0xFF55);
    
    let chip8 = run_program_default(&program);

    let expected = &[0x00, 0x1, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
                     0x0D, 0x0E, 0x0F];

    assert_eq!(expected, &chip8.memory[0x0..0x10]);
}

/// Tests that RegDump writes to the correct address
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn reg_dump_address() {
    // Fill the registers with the numbers 0 to 15, then dump them at address 255
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0x6505, 0x6606, 0x6707, 0x6808,
                           0x6909, 0x6A0A, 0x6B0B, 0x6C0C, 0x6D0D, 0x6E0E, 0x6F0F, 0xA0FF, 0xFF55);
    
    let chip8 = run_program_default(&program);

    let expected = &[0x00, 0x1, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
                     0x0D, 0x0E, 0x0F];

    assert_eq!(expected, &chip8.memory[0xFF..0x10F]);
}

/// Tests instruction RegLoad
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn reg_load() {
    // Fill the registers with the numbers 0 to 15, then loads values from address 0, which should
    // set all registers to 0
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0x6505, 0x6606, 0x6707, 0x6808,
                           0x6909, 0x6A0A, 0x6B0B, 0x6C0C, 0x6D0D, 0x6E0E, 0x6F0F, 0xFF65);

    let chip8 = run_program_default(&program);

    assert_eq!(&[0; 16], chip8.registers.get_registers());
}

/// Tests that RegLoad writes to the correct address
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn reg_load_address() {
    // Fill the registers with the numbers 0 to 15, dumps them to address 255, sets all registers to
    // 0 by loading from address 0, then loads from address 255
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0x6505, 0x6606, 0x6707, 0x6808,
                           0x6909, 0x6A0A, 0x6B0B, 0x6C0C, 0x6D0D, 0x6E0E, 0x6F0F, 0xA0FF, 0xFF65,
                           0xA000, 0xFF65, 0xA0FF, 0xFF65);
    
    let chip8 = run_program_default(&program);

    assert_eq!(&[0; 16], chip8.registers.get_registers());
}

/// Tests instruction SetIndex
#[test]
fn set_index() {
    let program = program!(0xAFFF);

    let chip8 = run_program_default(&program);

    assert_eq!(0xFFF, chip8.registers.index);
}

/// Tests instruction AddIndex
#[test]
fn add_index() {
    let program = program!(0x60FF, 0xF01E);

    let chip8 = run_program_default(&program);

    assert_eq!(0xFF, chip8.registers.index);
}

/// Tests instruction SetIndexChar
#[test]
fn set_index_char() {
    let program = program!(0x600F, 0xF029);

    let chip8 = run_program_default(&program);

    assert_eq!(::FONTSET_START as u16 + 0x4B, chip8.registers.index);
}

/// Tests instruction GetDelay
#[test]
fn get_delay() {
    // Sets the delay timer to 2 at cycle 2, and it runs for 3 cycles, so the delay timer should be
    // 1 (only one tick passes before the GetDelay instruction is executed)
    let program = program!(0x6002, 0xF015, 0xF107);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(1));
}

/// Tests instruction SetDelay
#[test]
fn set_delay() {
    // Sets the delay timer to 255
    // Because run_program updates the timers after running a cycle, the delay timer gets
    // decremented once, which means the delay timer should be 254 rather than 255
    let program = program!(0x60FF, 0xF015);

    let chip8 = run_program_default(&program);

    assert_eq!(0xFE, chip8.delay_timer);
}

/// Tests that WaitKey stores key correctly (correct register and value)
#[test]
fn wait_key() {
    let program = program!(0xF00A);

    // Uses KeyIO instead of Io
    let (chip8, _) = run_program::<KeyIO>(&program, None, None);

    assert_eq!(15, chip8.registers.get(0));
}

/// Tests that WaitKey correctly waits for a key to be pressed
#[test]
fn wait_key_delay() {
    // The implementation of Chip8IO for KeyIO will increment its counter until it reaches 10
    // When it reaches 10, it presses a key, ending the WaitKey instructions
    // This test makes sure the counter reaches 10
    let program = program!(0xF00A);

    // Uses KeyIO instead of Io
    let (_, io) = run_program::<KeyIO>(&program, None, None);

    assert_eq!(10, io.get_keys_counter);
}

/// Tests instruction SkipKey when the skip should happen
#[test]
fn skip_key_0() {
    let program = program!(0x6001, 0xE09E, 0x61FF, 0x6101);

    let keypresses = keypresses!(1 @ 0..3);

    let (chip8, _) = run_program::<Io>(&program, Some(keypresses), Some(3));

    assert_eq!(0x1, chip8.registers.get(1));
}

/// Tests instruction SkipKey when the skip should not happen
#[test]
fn skip_key_1() {
    let program = program!(0x6001, 0xE09E, 0x61FF, 0x6101);

    let keypresses = keypresses!();

    let (chip8, _) = run_program::<Io>(&program, Some(keypresses), Some(3));

    assert_eq!(0xFF, chip8.registers.get(1));
}

/// Tests instruction SkipNotKey when the skip should happen
#[test]
fn skip_not_key_0() {
    let program = program!(0x6001, 0xE0A1, 0x61FF, 0x6101);

    let keypresses = keypresses!();

    let (chip8, _) = run_program::<Io>(&program, Some(keypresses), Some(3));

    assert_eq!(0x1, chip8.registers.get(1));
}

/// Tests instruction SkipNotKey when the skip should not happen
#[test]
fn skip_not_key_1() {
    let program = program!(0x6001, 0xE0A1, 0x61FF, 0x6101);

    let keypresses = keypresses!(1 @ 0..3);

    let (chip8, _) = run_program::<Io>(&program, Some(keypresses), Some(3));

    assert_eq!(0xFF, chip8.registers.get(1));
}

/// Tests instruction SetSound
#[test]
fn set_sound() {
    // Sets the sound timer to 255
    // Because run_program updates the timers after running a cycle, the delay timer gets
    // decremented once, which means the sound timer should be 254 rather than 255
    let program = program!(0x60FF, 0xF018);

    let chip8 = run_program_default(&program);

    assert_eq!(0xFE, chip8.sound_timer);
}

/// Tests that `Chip8IO::play_sound` is called when the sound timer reaches 0
/// This test should panic because the implementation of `Chip8IO::play_sound` for `Io` panics when
/// called
#[test]
#[should_panic(expected = "playing sound")]
fn play_sound_0() {
    let program = program!(0x6001, 0xF018);

    run_program_default(&program);
}

/// Tests that `Chip8IO::play_sound` is not called when the sound timer doesn't reach 0
#[test]
fn play_sound_1() {
    let program = program!(0x6002, 0xF018);

    run_program_default(&program);
}

/// Tests instruction Draw
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn draw() {
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0xFF55, 0xD005);

    let chip8 = run_program_default(&program);
    let pixels = to_matrix(chip8.io.pixels(), ::SCREEN_WIDTH, ::SCREEN_HEIGHT);

    let row0 = &pixels[0][0..8];
    let row1 = &pixels[1][0..8];
    let row2 = &pixels[2][0..8];
    let row3 = &pixels[3][0..8];
    let row4 = &pixels[4][0..8];

    // Bitcoded 0
    let expected_row0 = &[false, false, false, false, false, false, false, false];
    // Bitcoded 1
    let expected_row1 = &[false, false, false, false, false, false, false, true];
    // Bitcoded 2
    let expected_row2 = &[false, false, false, false, false, false, true, false];
    // Bitcoded 3
    let expected_row3 = &[false, false, false, false, false, false, true, true];
    // Bitcoded 4
    let expected_row4 = &[false, false, false, false, false, true, false, false];

    assert_eq!(expected_row0, row0);
    assert_eq!(expected_row1, row1);
    assert_eq!(expected_row2, row2);
    assert_eq!(expected_row3, row3);
    assert_eq!(expected_row4, row4);
}

/// Tests that Draw sets VF to 1 when a pixel is flipped from set to unset
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn draw_flip_0() {
    let program = program!(0x6001, 0xF055, 0xD111, 0x6000, 0xF055, 0xD111);

    let chip8 = run_program_default(&program);

    assert_eq!(0x1, chip8.registers.get(0xF));
}

/// Tests that Draw sets VF to 0 when no pixel is flipped from set to unset
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn draw_flip_1() {
    let program = program!(0x6000, 0xF055, 0xD001, 0x6FFF, 0xD001);

    let chip8 = run_program_default(&program);

    assert_eq!(0x0, chip8.registers.get(0xF));
}

/// Tests that Draw draws to the correct location
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn draw_location() {
    // TODO: CONTINUE HERE:
    //       Make this test draw to the bottom right corner and update the expected_row* variables
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0xFF55, 0x6038, 0x611B, 0xD015);

    let chip8 = run_program_default(&program);
    let pixels = to_matrix(chip8.io.pixels(), ::SCREEN_WIDTH, ::SCREEN_HEIGHT);

    let row0 = &pixels[27][56..];
    let row1 = &pixels[28][56..];
    let row2 = &pixels[29][56..];
    let row3 = &pixels[30][56..];
    let row4 = &pixels[31][56..];

    // Bitcoded 0
    let expected_row0 = &[false, false, false, false, false, false, false, false];
    // Bitcoded 1
    let expected_row1 = &[false, false, false, false, false, false, false, true];
    // Bitcoded 2
    let expected_row2 = &[false, false, false, false, false, false, true, false];
    // Bitcoded 3
    let expected_row3 = &[false, false, false, false, false, false, true, true];
    // Bitcoded 4
    let expected_row4 = &[false, false, false, false, false, true, false, false];

    assert_eq!(expected_row0, row0);
    assert_eq!(expected_row1, row1);
    assert_eq!(expected_row2, row2);
    assert_eq!(expected_row3, row3);
    assert_eq!(expected_row4, row4);
}

/// Tests instruction ClearScreen
#[test]
#[cfg_attr(rustfmt, rustfmt_skip)]
fn clear_screen() {
    let program = program!(0x6000, 0x6101, 0x6202, 0x6303, 0x6404, 0xFF55, 0xD005, 0x00E0);

    let chip8 = run_program_default(&program);

    assert_eq!(vec![false; ::SCREEN_WIDTH * ::SCREEN_HEIGHT], chip8.io.pixels().to_vec());
}
