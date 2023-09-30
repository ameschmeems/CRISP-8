use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

// originally the chip-8 interpreter was located in ram 0x000 - 0x1ff, and expected programs to load right after
const START_ADDR: u16 = 0x200;
const FONT_ADDR: u16 = 0x050;

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
0x20, 0x60, 0x20, 0x20, 0x70, // 1
0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
0x90, 0x90, 0xF0, 0x10, 0x10, // 4
0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
0xF0, 0x10, 0x20, 0x40, 0x40, // 7
0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
0xF0, 0x90, 0xF0, 0x90, 0x90, // A
0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
0xF0, 0x80, 0x80, 0x80, 0xF0, // C
0xE0, 0x90, 0x90, 0x90, 0xE0, // D
0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Emu
{
	// program counter, stores info about which instruction to execute next
	pc: u16,
	ram: [u8; RAM_SIZE],
	screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
	keys: [bool; NUM_KEYS],
	// registers, numbered V0 to VF
	// VF is used as a flag register
	v_reg: [u8; NUM_REGS],
	// 16 bit register, used for indexing into ram
	i_reg: u16,
	// stack is not general purpose, only used when entering/exiting subroutines
	stack: [u16; STACK_SIZE],
	// stack pointer, keeps track of where the top currently is
	sp: u16,
	// delay timer register, counts down every cycle and performs an action when it hits 0
	dt: u8,
	// sound timer register, counts down every cycle, emits a noise when it hits 0
	st: u8
}

impl Emu
{
	pub fn new() -> Self
	{
		let mut new_emu = Self {
			pc: START_ADDR,
			ram: [0; RAM_SIZE],
			screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
			keys: [false; NUM_KEYS],
			v_reg: [0; NUM_REGS],
			i_reg: 0,
			sp: 0,
			stack: [0; STACK_SIZE],
			dt: 0,
			st: 0
		};

		// the fonts can be stored anywhere before 0x200, but putting it between 0x050 and 0x09f has become a popular convention in emulators
		new_emu.ram[(FONT_ADDR as usize)..((FONT_ADDR as usize) + FONTSET_SIZE)].copy_from_slice(&FONTSET);

		new_emu
	}

	pub fn reset(&mut self)
	{
		self.pc = START_ADDR;
		self.ram = [0; RAM_SIZE];
		self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
		self.keys = [false; NUM_KEYS];
		self.v_reg = [0; NUM_REGS];
		self.i_reg = 0;
		self.sp = 0;
		self.stack = [0; STACK_SIZE];
		self.dt = 0;
		self.st = 0;
		self.ram[(FONT_ADDR as usize)..((FONT_ADDR as usize) + FONTSET_SIZE)].copy_from_slice(&FONTSET);
	}

	fn push(&mut self, val: u16)
	{
		self.stack[self.sp as usize] = val;
		self.sp += 1;
	}

	fn pop(&mut self) -> u16
	{
		self.sp -= 1;
		self.stack[self.sp as usize]
	}

	pub fn tick(&mut self)
	{
		// get opcode
		let op =  self.fetch();
		// decode and execute operation
		self.execute(op);
	}

	// unlike the regular ticks which operates once every cpu cycle, the timers are modified once every frame, thus needing a seperate function
	pub fn tick_timers(&mut self)
	{
		if self.dt > 0
		{
			self.dt -= 1;
		}

		if self.st > 0
		{
			self.st -= 1;
			if self.st == 0
			{
				// sound here
				println!("Boop!");
			}
		}
	}

	fn fetch(&mut self) -> u16
	{
		let higher_byte = self.ram[self.pc as usize] as u16;
		let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
		let op = (higher_byte << 8) | lower_byte;
		self.pc += 2;
		op
	}

	pub fn execute(&mut self, op: u16)
	{
		let digit1 = (op & 0xf000) >> 12;
		let digit2 = (op & 0x0f00) >> 8;
		let digit3 = (op & 0x00f0) >> 4;
		let digit4 = op & 0x000f;

		match (digit1, digit2, digit3, digit4)
		{
			// opcode 0000: NOP
			(0x0, 0x0, 0x0, 0x0) => return,
			// opcode 00e0: CLS
			(0x0, 0x0, 0xe, 0x0) => { self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT]; },
			// opcode 1NNN: JMP NNN
			(0x1, _, _, _) => {
				let nnn = op & 0x0fff;
				self.pc = nnn;
			},
			// opcode 2NNN: CALL NNN
			(0x2, _, _, _) => {
				self.push(self.pc);
				let nnn = op & 0x0fff;
				self.pc = nnn;
			},
			// opcode 00ee: RET
			(0x0, 0x0, 0xe, 0xe) => {
				let ret_addr = self.pop();
				self.pc = ret_addr;
			},
			// opcode 3XNN: SKIP VX == NN
			(0x3, _, _, _) => {
				let x = digit2;
				let nn = (op & 0x00ff) as u8;
				if self.v_reg[x as usize] == nn
				{
					self.pc += 2;
				}
			},
			// opcode 4XNN: SKIP VX != NN
			(0x4, _, _, _) => {
				let x = digit2;
				let nn = (op & 0x00ff) as u8;
				if self.v_reg[x as usize] != nn
				{
					self.pc += 2;
				}
			},
			// opcode 5XY0: SKIP VX == VY
			(0x5, _, _, 0x0) => {
				let x = digit2;
				let y = digit3;
				if self.v_reg[x as usize] == self.v_reg[y as usize]
				{
					self.pc += 2;
				}
			},
			// opcode 9XY0: SKIP VX != VY
			(0x9, _, _, 0x0) => {
				let x = digit2;
				let y = digit3;
				if self.v_reg[x as usize] != self.v_reg[y as usize]
				{
					self.pc += 2;
				}
			},
			// opcode 6XNN: VX = NN
			(0x6, _, _, _) => {
				let x = digit2;
				let nn = (op & 0x00ff) as u8;
				self.v_reg[x as usize] = nn;
			},
			// opcode 7XNN: VX += NN
			// note - this doesnt set the carry flag if overflow happens
			(0x7, _, _, _) => {
				let x = digit2;
				let nn = (op & 0x00ff) as u8;
				self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
			},
			// opcode 8XY0: VX = VY
			(0x8, _, _, 0x0) => {
				let x = digit2;
				let y = digit3;
				self.v_reg[x as usize] = self.v_reg[y as usize];
			},
			// opcode 8XY1: VX |= VY
			(0x8, _, _, 0x1) => {
				let x = digit2;
				let y = digit3;
				self.v_reg[x as usize] |= self.v_reg[y as usize];
			},
			// opcode 8XY2: VX &= VY
			(0x8, _, _, 0x2) => {
				let x = digit2;
				let y = digit3;
				self.v_reg[x as usize] &= self.v_reg[y as usize];
			},
			// opcode 8XY3: VX ^= VY
			(0x8, _, _, 0x3) => {
				let x = digit2;
				let y = digit3;
				self.v_reg[x as usize] ^= self.v_reg[y as usize];
			},
			// opcode 8XY4: VX += VY
			// note - unlike 7XNN this does set the carry flag in case of overflow
			(0x8, _, _, 0x4) => {
				let x = digit2;
				let y = digit3;
				
				let (new_vx, carry) = self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);
				let new_vf = if carry { 1 } else { 0 };
				self.v_reg[x as usize] = new_vx;
				self.v_reg[0xf] = new_vf;
			},
			// opcode 8XY5: VX -= VY, sets carry flag to 0 if it underflows, or otherwise to 1
			(0x8, _, _, 0x5) => {
				let x = digit2;
				let y = digit3;
				
				let (new_vx, borrow) = self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);
				let new_vf = if borrow { 0 } else { 1 };
				self.v_reg[x as usize] = new_vx;
				self.v_reg[0xf] = new_vf;
			},
			// opcode 8XY7: VX = VY - VX, sets carry flag same as 8XY5
			(0x8, _, _, 0x7) => {
				let x = digit2;
				let y = digit3;

				let (new_vx, borrow) = self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);
				let new_vf = if borrow { 0 } else { 1 };
				self.v_reg[x as usize] = new_vx;
				self.v_reg[0xf] = new_vf;
			},
			// opcode 8XY6: VX >>= 1, sets flag to the lost bit
			// note - older implementations first set VX to VY, but newer implementations ignore the Y value completely
			(0x8, _, _, 0x6) => {
				let x = digit2;
				let lost_bit = self.v_reg[x as usize] & 1;
				self.v_reg[x as usize] >>= 1;
				self.v_reg[0xf] = lost_bit;
			},
			// opcode 8XYE: VX <<= 1. sets flag to the lost bit
			// note - same as 8XY6
			(0x8, _, _, 0xe) => {
				let x = digit2;
				let lost_bit = (self.v_reg[x as usize] >> 7) & 1;
				self.v_reg[x as usize] <<= 1;
				self.v_reg[0xf] = lost_bit;
			},
			// opcode ANNN: I = NNN
			(0xa, _, _, _) => {
				let nnn = op & 0x0fff;
				self.i_reg = nnn;
			},
			// opcode BNNN: JMP NNN + V0
			// note - some newer implementations (likely unintentionally) treat the second digit as X, resulting in JMP XNN + VX
			// this is not a common operation, so sticking with the older version should be fine
			(0xb, _, _, _) => {
				let nnn = op & 0x0fff;
				self.pc = self.v_reg[0x0] as u16 + nnn;
			},
			// opcode CXNN: rand() & NN
			(0xc, _, _, _) => {
				let x = digit2;
				let nn = (op & 0x00ff) as u8;
				let random_num: u8 = random();
				self.v_reg[x as usize] = random_num & nn;
			},
			// opcode DXYN: DRAW
			(0xd, _, _, _) => {
				self.draw(digit2, digit3, digit4);
			},
			// opcode EX9E: SKIP if key specified in VX is currently pressed
			(0xe, _, 0x9, 0xe) => {
				let x = digit2;
				let vx = self.v_reg[x as usize];
				if self.keys[vx as usize]
				{
					self.pc += 2;
				}
			},
			// opcode EXA1: SKIP if key specified in VX is not currently pressed
			(0xe, _, 0xa, 0x1) => {
				let x = digit2;
				let vx = self.v_reg[x as usize];
				if !self.keys[vx as usize]
				{
					self.pc += 2;
				}
			},
			// opcode FX07: set VX to current value of delay timer
			(0xf, _, 0x0, 0x7) => {
				let x = digit2;
				self.v_reg[x as usize] = self.dt;
			},
			// opcode FX15: set delay timer to VX
			(0xf, _, 0x1, 0x5) => {
				let x = digit2;
				self.dt = self.v_reg[x as usize];
			},
			// opcode FX18: set sound timer to VX
			(0xf, _, 0x1, 0x8) => {
				let x = digit2;
				self.st = self.v_reg[x as usize];
			},
			// opcode FX1E: I += VX, sets flag to 1 if I "overflows" from 0x0fff to 0x1000
			// note - not all implementations set the flag, but there's no harm in doing it (and some games might rely on it)
			(0xf, _, 0x1, 0xe) => {
				let x = digit2;
				self.i_reg += self.v_reg[x as usize] as u16;
				let new_vf = if self.i_reg >= 0x1000 { 1 } else { 0 };
				self.v_reg[0xf] = new_vf;
			},
			// opcode FX0A: Blocks continuation of program until a key press is detected, then stores it into VX
			// note - in case of multiple pressed keys, take the lowest indexed one
			(0xf, _, 0x0, 0xa) => {
				let x = digit2 as usize;
				let mut pressed = false;
				for i in 0..self.keys.len()
				{
					if self.keys[i]
					{
						self.v_reg[x as usize] = i as u8;
						pressed = true;
						break ;
					}
				}
				if !pressed
				{
					// reset instruction
					self.pc -= 2;
				}
			},
			// opcode FX29: set I to the font address of character in VX
			// note - if VX is bigger than 0x0f, only take the last digit
			(0xf, _, 0x2, 0x9) => {
				let x = digit2;
				let c = self.v_reg[x as usize] as u16;
				let c = c & 0x0f;
				self.i_reg = FONT_ADDR + c * 5;
			},
			// opcode FX33: puts 3 decimal digits of the number in VX at I, I + 1 and I + 2 respectively
			(0xf, _, 0x3, 0x3) => {
				let x = digit2;
				let vx = self.v_reg[x as usize];
				let digit1 = vx / 100;
				let digit2 = (vx - digit1 * 100) / 10;
				let digit3 = vx % 10;
				self.ram[self.i_reg as usize] = digit1;
				self.ram[(self.i_reg + 1) as usize] = digit2;
				self.ram[(self.i_reg + 2) as usize] = digit3;
			},
			// opcode FX55: store registers V0 - VX to memory, in incremental addresses starting from I
			// note - originally I was modified in the process, but modern implementations leave it intact
			(0xf, _, 0x5, 0x5) => {
				let x = digit2;
				for i in 0..(x + 1) as u32
				{
					self.ram[(self.i_reg as u32 + i) as usize] = self.v_reg[i as usize];
				}
			},
			// opcode FX65: load values stored at I - I + X, and store them in registers V0 - VX
			(0xf, _, 0x6, 0x5) => {
				let x = digit2;
				for i in 0..(x + 1) as u32
				{
					self.v_reg[i as usize] = self.ram[(self.i_reg as u32 + i) as usize];
				}
			}
			(_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
		}
	}

	fn draw(&mut self, x: u16, y: u16, n: u16)
	{
		// the original x and y coordinates wrap around, however if the sprite goes off screen, it should be clipped instead of wrapped
		let x_coord = self.v_reg[x as usize] as usize % SCREEN_WIDTH;
		let y_coord = self.v_reg[y as usize] as usize % SCREEN_HEIGHT;
		self.v_reg[0xf as usize] = 0;
		// if any pixels are flipped off, we need to write to the flag register
		let mut flip = false;
		for row in 0..n
		{
			// sprite is stored at addr specified by the I register
			let row_addr = self.i_reg + row;
			let pixels = self.ram[row_addr as usize];
			for col in 0..8
			{
				if y_coord + row as usize >= SCREEN_HEIGHT
				{
					break ;
				}
				if (pixels & (0b1000_0000 >> col)) != 0
				{
					
					let x = x_coord + col as usize;
					let y = y_coord + row as usize;
					if x >= SCREEN_WIDTH
					{
						break ;
					}
					let pixel_index = x + SCREEN_WIDTH * y;
					// set flip to true if pixel was already on
					flip |= self.screen[pixel_index];
					// flip pixel
					self.screen[pixel_index] ^= true;
				}
			}
		}
		if flip
		{
			self.v_reg[0xf as usize] = 1;
		}
	}

	pub fn get_screen(&self) -> &[bool]
	{
		&self.screen
	}

	pub fn load(&mut self, data: &[u8])
	{
		let start = START_ADDR as usize;
		let end = START_ADDR as usize + data.len();
		self.ram[start..end].copy_from_slice(data);
	}

	pub fn keypress(&mut self, key: usize, down: bool)
	{
		self.keys[key] = down;
	}
}