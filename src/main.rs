use minifb::{MouseButton, MouseMode, Window, WindowOptions};

fn main() {
    let width = 200;
    let height = 200;
    let white = 0x00FFFFFF; // 16777215 Decimal value
    let red = 0x00FF0000;
    let black = 0x00080808;
    let mut buffer: Vec<u32> = vec![0; width * height];

    let mut window = Window::new("Moving Box", width, height, WindowOptions::default()).unwrap();

    let mut x = 0;
    buffer.fill(white);
    let mut was_pressed = false;

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        let is_pressed = window.get_mouse_down(MouseButton::Left);
        for i in 0..20 {
            for j in 0..20 {
                let idx = (j * width + (x)) as usize;
                if idx < buffer.len() {
                    if i == 1 {
                        buffer[idx] = white;
                    }
                    if x == 0 {
                        buffer.fill(white);
                    }
                    buffer[idx + i] = red;
                }
            }
        }

        if is_pressed && !was_pressed {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                let (x, y) = (mx as usize, my as usize);

                let idx = y * width + x;

                println!("CLICKED ON: {}", buffer[idx]);
                if buffer[idx] != white {
                    println!("CLICKED ON SOMETHING");
                }
            }
        }

        x = (x + 1) % (width - 20); // move square horizontally

        for i in 0..10 {
            let start_index = (100 + i) * width + 0;
            let end_index = start_index + 100;
            buffer[start_index..end_index].fill(black);
        }

        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
