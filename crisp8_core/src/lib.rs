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

		// the fonts can be stored anywhere before 0x200, but putting it between 0x050 and 0x1ff has become a popular convention in emulators
		new_emu.ram[0x050..0x1ff].copy_from_slice(&FONTSET);

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
		self.ram[0x050..0x1ff].copy_from_slice(&FONTSET);
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
			(_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
		}
	}
}