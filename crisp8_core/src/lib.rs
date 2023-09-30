pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

// originally the chip-8 interpreter was located in ram 0x000 - 0x1ff, and expected programs to load right after
const START_ADDR: u16 = 0x200;

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
		new_emu.ram[0x050..0x0a0].copy_from_slice(&FONTSET);

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
		self.ram[0x050..0x0a0].copy_from_slice(&FONTSET);
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
				self.v_reg[x as usize] += nn;
			},
			// opcode ANNN: I = NNN
			(0xa, _, _, _) => {
				let nnn = op & 0x0fff;
				self.i_reg = nnn;
			}
			// opcode DXYN: DRAW
			(0xd, _, _, _) => {
				self.draw(digit2, digit3, digit4);
			},
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
}