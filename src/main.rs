use glfw::{fail_on_errors, Action, Context, Key, Window};

// Tileset variables
const ORIGINAL_TILE_SIZE: u32 = 16;
const SCALE: u32 = 3;

// Game Screen Variables
const MAX_SCREEN_COLUMNS: u32 = 16;
const MAX_SCREEN_ROWS: u32 = 12;

// Derived Constants
const TILE_SIZE: u32 = ORIGINAL_TILE_SIZE * SCALE;
const SCREEN_WIDTH: u32 = TILE_SIZE * MAX_SCREEN_COLUMNS;
const SCREEN_HEIGHT: u32 = TILE_SIZE * MAX_SCREEN_ROWS;

#[tokio::main]
async fn main() {
    let mut glfw = glfw::init(fail_on_errors).expect("Failed to create a glfw");

    let (mut window, events) = glfw
        .create_window(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            "My First Game From Scratch",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create window.");

    window.set_resizable(false);
    window.set_key_polling(true);
    window.make_current();

    let mut state = State::new(&mut window).await;

    while !state.window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    state.window.set_should_close(true)
                }
                _ => {}
            }
        }

        match state.render() {
            Ok(_) => {}
            Err(e) => eprintln!("Failed to render: {e}"),
        }
        state.window.swap_buffers();
    }
}
