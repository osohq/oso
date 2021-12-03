use nannou::prelude::*;
use nannou_osc as osc;

use std::{net::SocketAddr, str::FromStr, time::Duration};

const BACKTRACK_FRAME_DURATION: u64 = 10;
const PACKETS_PER_UPDATE: usize = 100;

// OSC client configuration
const OSC_LISTEN_PORT: u16 = 9100;
const OSC_SOURCE: &str = "127.0.0.1:9000";

// UI dimensions & other variables
const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const LINE_WIDTH: f32 = 2.0;

// model to record visualizer simulation state between redraws
struct Model {
    source: SocketAddr,
    receiver: osc::Receiver,
    current_depth: u32,
    backtrack: bool,
    backtrack_last_frame: u64,
}

// instantiate a fresh Model w/ receiver & packet buffer
fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::Rate {
        update_interval: Duration::new(0, 50_000_000),
    });

    // build our main window instance w/ title & dimensions
    app.new_window()
        .title("oso visualizer")
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();

    // instantiate model state w/ OSC parameters
    let source = SocketAddr::from_str(OSC_SOURCE).unwrap();
    let receiver = osc::receiver(OSC_LISTEN_PORT).unwrap();

    Model {
        source,
        receiver,
        current_depth: 0,
        backtrack: false,
        backtrack_last_frame: 0,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // gather relevant incoming packets into our buffer
    let mut packets = vec![];
    for (packet, addr) in model.receiver.try_iter() {
        if addr == model.source {
            packets.push(packet);
        }
    }

    while packets.len() > PACKETS_PER_UPDATE {
        packets.remove(0);
    }

    if model.backtrack
        && (app.elapsed_frames() - model.backtrack_last_frame) > BACKTRACK_FRAME_DURATION
    {
        model.backtrack = false;
        model.backtrack_last_frame = 0;
    }

    // forgive this else ladder :scream:
    let max_depth: i32 = packets
        .iter()
        .map(|p| {
            if let osc::Packet::Message(message) = p {
                // love to mutate other separate state while looping in a map!
                if message.addr.contains("backtrack") && !model.backtrack {
                    model.backtrack = true;
                    model.backtrack_last_frame = app.elapsed_frames();
                }

                if let Some(args) = &message.args {
                    if let osc::Type::Int(depth) = args[0] {
                        depth
                    } else {
                        0
                    }
                } else {
                    0
                }
            } else {
                0
            }
        })
        .max()
        .unwrap_or(1);

    model.current_depth = max_depth as u32;
}

fn main() {
    nannou::app(model).update(update).run();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    let background = if model.backtrack { CRIMSON } else { WHITE };
    let foreground = if model.backtrack { WHITE } else { BLACK };
    draw.background().color(background);

    for depth in 0..model.current_depth {
        let s = WIDTH - (30 * depth as u32);
        draw.rect()
            .no_fill()
            .stroke_weight(LINE_WIDTH)
            .stroke_color(foreground)
            .w_h(s as f32, s as f32)
            .x_y(0.0, 0.0);
    }

    draw.to_frame(app, &frame).unwrap();
}
