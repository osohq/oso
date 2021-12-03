use nannou::{find_folder::KidsDepth, prelude::*};
use nannou_conrod as ui;
use nannou_conrod::prelude::*;
use nannou_osc as osc;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
};

// OSC client configuration
const OSC_LISTEN_PORT: u16 = 9100;
const OSC_SOURCE: &str = "127.0.0.1:9000";

// UI dimensions & other variables
const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const LINE_WIDTH: f32 = 1.0;

// model to record visualizer simulation state between redraws
struct Model {
    source: SocketAddr,
    receiver: osc::Receiver,
    packets: Vec<osc::Packet>,
}

// instantiate a fresh Model w/ receiver & packet buffer
fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::RefreshSync);

    let w_id = app
        .new_window()
        .title("oso visualizer")
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();

    let source = SocketAddr::from_str(OSC_SOURCE).unwrap();
    let receiver = osc::receiver(OSC_LISTEN_PORT).unwrap();
    let packets = vec![];

    Model {
        source,
        receiver,
        packets,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // gather relevant incoming packets into our buffer
    for (packet, addr) in model.receiver.try_iter() {
        if addr == model.source {
            model.packets.push(packet);
        }
    }

    let packets_per_tick = 20;
    while model.packets.len() > packets_per_tick {
        model.packets.remove(0);
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn view(app: &App, model: &Model, frame: Frame) {
    println!("in the view fn");
    let draw = app.draw();

    draw.background().color(WHITE);

    let max_depth: i32 = model
        .packets
        .iter()
        .map(|p| {
            if let osc::Packet::Message(message) = p {
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

    for depth in 0..max_depth {
        let s = WIDTH - (30 * depth as u32);
        draw.rect()
            .no_fill()
            .stroke_weight(LINE_WIDTH)
            .w_h(s as f32, s as f32)
            .x_y(0.0, 0.0);
    }

    draw.to_frame(app, &frame).unwrap();
}
