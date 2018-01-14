mod chip8;

fn main() {

      let mut chip = chip8::Chip::new();

      chip.load_rom("roms/rand.rom");
      chip.run();
      chip.dump();
}