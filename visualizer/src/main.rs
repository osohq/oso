use nannou::prelude::*;

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
    draw.rect()
        .no_fill()
        .stroke(BLACK)
        .stroke_weight(LINE_WIDTH)
        .w_h(200.0, 200.0)
        .x_y(0.0, 0.0);

    draw.to_frame(app, &frame).unwrap();
}
