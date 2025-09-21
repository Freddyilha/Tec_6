use minifb::{Window, WindowOptions};

fn main() {
    let width = 200;
    let height = 200;
    let start_index = 100 * width + 0;
    let end_index = start_index + 100;
    let white = 0x00FFFFFF;
    let red = 0x00FF0000;
    let black = 0x00080808;
    let mut buffer: Vec<u32> = vec![0; width * height];

    let mut window = Window::new(
        "Moving Box"
        , width
        , height
        , WindowOptions::default()
    ).unwrap();

    let mut x = 0;
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(white);

        for i in 0..20 {
            for j in 0..20 {
                let idx = (j * width + (x + i)) as usize;
                if idx < buffer.len() {
                    buffer[idx] = red;
                }
            }
        }

        x = (x + 1) % (width - 20); // move square horizontally

        buffer[start_index..end_index].fill(black);

        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
