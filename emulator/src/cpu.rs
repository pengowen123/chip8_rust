//! Emulation of the Chip-8 CPU

use rand;

use super::Chip8;
use errors::*;
use interpreter::interpret_instruction;
use instruction::Instruction;
use fontset::FONTSET_START;
use utils;

impl Chip8 {
    /// Runs a CPU cycle, calling the input function to update the internal key state
    /// Requires a type that implements `Chip8IO` to do I/O (see `Chip8IO` for more)
    pub fn cycle<T: ::Chip8IO>(&mut self, mut io: &mut T) -> Result<()> {
        let memory = &mut self.memory;
        let stack = &mut self.stack;
        // Registers
        let registers = &mut self.registers;
        let pc = registers.program_counter;
        // Used for indexing
        let pc_index = pc as usize;

        // If the program counter is out of bounds, end the program
        if memory.get(pc_index + 1).is_none() {
            self.program_ended = true;
            return Ok(());
        }

        // Load the opcode from memory
        let opcode = (memory[pc_index] as u16) << 8 | memory[pc_index + 1] as u16;
        // Try to convert the opcode to an instruction
        let instruction = interpret_instruction(opcode)
            .chain_err(|| format!("Invalid opcode at address {}", pc))?;

        if self.log.is_enabled() {
            info!("OPCODE: 0x{:04X}", opcode);
        }

        // Not all instructions require incrementing the program counter
        // This is set to false by those instructions to prevent the increment
        let mut increment_pc = true;

        self.io.set_keys(io.get_keys());

        match instruction {
            Instruction::Return => {
                if let Some(addr) = stack.pop() {
                    registers.program_counter = addr;
                }
            }
            Instruction::Goto(addr) => {
                if addr as usize >= ::MEMORY {
                    bail!(ErrorKind::InvalidAddress(addr as usize, "Goto"));
                }
                registers.program_counter = addr;
                increment_pc = false;
            }
            Instruction::Call(addr) => {
                if addr as usize >= ::MEMORY {
                    bail!(ErrorKind::InvalidAddress(addr as usize, "Call"));
                }

                registers.program_counter = addr;
                stack.push(pc);
                increment_pc = false;
            }
            Instruction::OffsetGoto(addr) => {
                let v0 = registers.get_u16(0);

                if (v0 + addr) as usize >= ::MEMORY {
                    bail!(ErrorKind::InvalidAddress(addr as usize, "OffsetGoto"));
                }

                registers.program_counter = addr + v0;
                increment_pc = false;
            }
            Instruction::SetConst(x, n) => registers.set(x, n),
            Instruction::AddConst(x, n) => {
                let val = registers.get(x).wrapping_add(n);
                registers.set(x, val);
            }
            Instruction::Move(x, y) => {
                let val = registers.get(y);
                registers.set(x, val);
            }
            Instruction::BitOr(x, y) => {
                let val = registers.get(x) | registers.get(y);
                registers.set(x, val);
            }
            Instruction::BitAnd(x, y) => {
                let val = registers.get(x) & registers.get(y);
                registers.set(x, val);
            }
            Instruction::BitXor(x, y) => {
                let val = registers.get(x) ^ registers.get(y);
                registers.set(x, val);
            }
            Instruction::Shr(x_id) => {
                let x = registers.get(x_id);
                let val = x >> 1;
                registers.set(x_id, val);

                // Set VF to the least significant bit of VX
                registers.set(0xF, x & 1);
            }
            Instruction::Shl(x_id) => {
                let x = registers.get(x_id);
                let val = x << 1;
                registers.set(x_id, val);

                // Set VF to the most significant bit of VX
                registers.set(0xF, (x & 0x80) >> 7);
            }
            Instruction::Add(x_id, y) => {
                let x = registers.get(x_id);
                let y = registers.get(y);
                registers.set(x_id, x.wrapping_add(y));

                // Set VF to 1 if a carry happened, 0 otherwise
                registers.set(0xF, x.checked_add(y).is_none() as u8);
            }
            Instruction::Sub(x_id, y) => {
                let x = registers.get(x_id);
                let y = registers.get(y);
                registers.set(x_id, x.wrapping_sub(y));

                // Set VF to 1 if a borrow happened, 0 otherwise
                registers.set(0xF, x.checked_sub(y).is_none() as u8);
            }
            Instruction::InverseSub(x_id, y) => {
                let x = registers.get(x_id);
                let y = registers.get(y);
                registers.set(x_id, y.wrapping_sub(x));

                // Set VF to 1 if a borrow happened, 0 otherwise
                registers.set(0xF, y.checked_sub(x).is_none() as u8);
            }
            Instruction::Rand(x, n) => {
                registers.set(x, rand::random::<u8>() & n);
            }
            Instruction::BCD(a) => {
                let a = registers.get(a);
                let i = registers.index as usize;

                if i + 2 >= memory.len() {
                    bail!(ErrorKind::InvalidAddress(i, "BCD"));
                }

                memory[i..i + 3].copy_from_slice(&utils::bcd(a));
            }
            Instruction::SkipEqConst(x, n) => {
                if registers.get(x) == n {
                    registers.program_counter += 2;
                }
            }
            Instruction::SkipNeqConst(x, n) => {
                if registers.get(x) != n {
                    registers.program_counter += 2;
                }
            }
            Instruction::SkipEq(x, y) => {
                if registers.get(x) == registers.get(y) {
                    registers.program_counter += 2;
                }
            }
            Instruction::SkipNeq(x, y) => {
                if registers.get(x) != registers.get(y) {
                    registers.program_counter += 2;
                }
            }
            Instruction::RegDump(x) => {
                let i = registers.index as usize;
                let x = x as usize;

                if i + x >= memory.len() {
                    bail!(ErrorKind::InvalidAddress(i, "RegDump"));
                }

                memory[i..i + x + 1].copy_from_slice(&registers.get_registers()[..x + 1]);
            }
            Instruction::RegLoad(x) => {
                let i = registers.index as usize;
                let x = x as usize;

                if i + x >= memory.len() {
                    bail!(ErrorKind::InvalidAddress(i, "RegLoad"));
                }

                registers.get_mut_registers()[..x + 1].copy_from_slice(&memory[i..i + x + 1]);
            }
            Instruction::SetIndex(addr) => registers.index = addr,
            Instruction::AddIndex(addr) => registers.index += registers.get_u16(addr),
            Instruction::SetIndexChar(x) => {
                let x = registers.get_u16(x);
                // Only values 0 through 15 are valid
                if x > 15 {
                    bail!(ErrorKind::UnknownCharacter(x as u8));
                }
                registers.index = FONTSET_START as u16 + 5 * x;
            }
            Instruction::GetDelay(x) => registers.set(x, self.delay_timer),
            Instruction::SetDelay(x) => self.delay_timer = registers.get(x),
            Instruction::WaitKey(x) => {
                let key = self.io.wait_key(io);
                registers.set(x, key);
            }
            Instruction::SkipKey(x) => {
                let x = registers.get(x);

                // Only values 0 to 15 are valid
                if x > 15 {
                    bail!(ErrorKind::UnknownKey(x, "SkipKey"));
                }

                if self.io.is_key_pressed(x) {
                    registers.program_counter += 2;
                }
            }
            Instruction::SkipNotKey(x) => {
                let x = registers.get(x);

                // Only values 0 to 15 are valid
                if x > 15 {
                    bail!(ErrorKind::UnknownKey(x, "SkipNotKey"));
                }

                if !self.io.is_key_pressed(x) {
                    registers.program_counter += 2;
                }
            }
            Instruction::SetSound(x) => self.sound_timer = registers.get(x),
            Instruction::Draw(x, y, height) => {
                let x = registers.get(x);
                let y = registers.get(y);

                let index = registers.index;
                // Set VF to 0, will be set to 1 later if a pixel is unset (used for collision
                // detection)
                registers.set(0xF, 0);

                for line in 0..height {
                    let i = (index + line as u16) as usize;

                    if i >= memory.len() {
                        bail!(ErrorKind::InvalidAddress(i, "Draw"));
                    }

                    // Iterator through each bit in the line
                    for bit in 0..8 {
                        // Each bit is a pixel
                        let mem_pixel = memory[i] & (128 >> bit);

                        let pixel_x = (x + bit) as usize;
                        let pixel_y = (y + line) as usize;

                        let pixel_index = pixel_x + pixel_y * ::SCREEN_WIDTH;

                        if pixel_x >= ::SCREEN_WIDTH || pixel_y >= ::SCREEN_HEIGHT {
                            bail!(ErrorKind::PixelOutOfBounds(pixel_x, pixel_y));
                        }

                        let screen_pixel = self.io.get_mut_pixel(pixel_index);

                        // If the pixel is on, and the new value is off, set VF
                        if *screen_pixel && mem_pixel == 0 {
                            registers.set(0xF, 1);
                        }

                        *screen_pixel = mem_pixel > 0;
                    }
                }

                self.io.set_draw_flag();
            }
            Instruction::ClearScreen => self.io.clear_screen(),
        }

        // Draw the screen
        if self.io.draw_flag() {
            io.draw(self.io.pixels());
        }

        // Increment the program counter
        if increment_pc {
            registers.program_counter += 2;
        }

        Ok(())
    }
}
