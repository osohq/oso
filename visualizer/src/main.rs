use nannou::{draw::properties::SetStroke, prelude::*};

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const LINE_WIDTH: f32 = 1.0;

fn main() {
    nannou::sketch(view).size(WIDTH, HEIGHT).run();
}

fn view(app: &App, frame: Frame) {
    app.set_loop_mode(LoopMode::loop_once());
    let draw = app.draw();

    draw.background().color(WHITE);

    for depth in 1..10 {
        let s = WIDTH - (30 * depth);
        draw.rect()
            .no_fill()
            .stroke_weight(LINE_WIDTH)
            .w_h(s as f32, s as f32)
            .x_y(0.0, 0.0);
    }

    draw.to_frame(app, &frame).unwrap();
}
