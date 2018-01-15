extern crate glium;
extern crate glium_sdl2;
extern crate sdl2;

mod chip8;
use glium_sdl2::DisplayBuild;
use glium::Surface;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

fn main() {

      let sdl_ctx = sdl2::init().unwrap();
      let video_subsys = sdl_ctx.video().unwrap();

      let gl_attr = video_subsys.gl_attr();
      gl_attr.set_context_profile(GLProfile::Core);
      gl_attr.set_context_version(3, 3);

      let window = video_subsys.window("CHIP-8", 800, 400)
            .build_glium()
            .unwrap();

      let mut ev_pump = sdl_ctx.event_pump().unwrap();
      'running: loop {
            let mut target = window.draw();
            //target.clear_color(0.2, 0.3, 0.3, 1.0);
            target.finish().unwrap();

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
      chip.run();
      chip.dump();
}