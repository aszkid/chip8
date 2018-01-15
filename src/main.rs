extern crate sdl2;
mod chip8;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

fn main() {

      let sdl_ctx = sdl2::init().unwrap();
      let video_subsys = sdl_ctx.video().unwrap();

      let gl_attr = video_subsys.gl_attr();
      gl_attr.set_context_profile(GLProfile::Core);
      gl_attr.set_context_version(3, 3);

      let window = video_subsys.window("CHIP-8", 800, 600)
            .opengl()
            .build()
            .unwrap();

      let gl_ctx = window.gl_create_context().unwrap();
      debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
      debug_assert_eq!(gl_attr.context_version(), (3, 3));

      let mut ev_pump = sdl_ctx.event_pump().unwrap();
      'running: loop {
            window.gl_swap_window();
            for ev in ev_pump.poll_iter() {
                  match ev {
                        Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                              break 'running
                        },
                        _ => {}
                  }
            }
      }

      let mut chip = chip8::Chip::new();

      chip.load_rom("roms/skip.rom");
      //chip.run();
      chip.dump();
}