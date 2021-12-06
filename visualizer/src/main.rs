use nannou::prelude::*;
use nannou_osc as osc;

use std::{net::SocketAddr, str::FromStr};

// constants which ultimately affect how jittery the UI is
const BACKTRACK_FRAME_DURATION: u64 = 60_000;
const PACKETS_PER_UPDATE: usize = 100;

// OSC client configuration
const OSC_LISTEN_PORT: u16 = 9101;
const OSC_SOURCE: &str = "127.0.0.1:9000";

// UI dimensions & other variables
const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;
const LINE_WIDTH: f32 = 1.5;

// model to record visualizer simulation state between redraws
struct Model {
    source: SocketAddr,
    receiver: osc::Receiver,

    current_depth: u32,
    max_goals: u32,

    backtrack: bool,
    backtrack_last_frame: u64,
}

// instantiate a fresh Model w/ receiver & packet buffer
fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::refresh_sync());

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
        max_goals: 0,
        backtrack: false,
        backtrack_last_frame: 0,
    }
}

fn unwrap_osc_args(args: Vec<osc::Type>) -> Vec<i32> {
    args.iter()
        .map(|a| match a {
            osc::Type::Int(v) => *v,
            _ => 0i32,
        })
        .collect()
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

    // ugly code to calculate max_goals and current_depth
    if !packets.is_empty() {
        let stack_and_goal_depths: Vec<(i32, i32)> = packets
            .iter()
            .map(|packet| {
                if let osc::Packet::Message(message) = packet {
                    // love to mutate other separate state while looping in a map!
                    if message.addr.contains("backtrack") && !model.backtrack {
                        model.backtrack = true;
                        model.backtrack_last_frame = app.elapsed_frames();
                    }

                    if message.args.is_some() {
                        let args = message.args.as_ref().unwrap();
                        let args = unwrap_osc_args(args.clone());
                        (args[0], args[1])
                    } else {
                        (1i32, 1i32)
                    }
                } else {
                    (1i32, 1i32)
                }
            })
            .collect();

        let cloned_values = stack_and_goal_depths.clone();
        let max_depth = stack_and_goal_depths
            .iter()
            .map(|(depth, _)| depth)
            .max()
            .unwrap_or(&1);

        let max_goals = cloned_values
            .iter()
            .map(|(_, goals)| goals)
            .max()
            .unwrap_or(&1);

        model.current_depth = *max_depth as u32;
        model.max_goals = *max_goals as u32;
    }
}

fn main() {
    nannou::app(model).update(update).run();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw().scale((app.time * 0.05).sin());

    let background = if model.backtrack { CRIMSON } else { WHITE };
    let foreground = if model.backtrack { WHITE } else { BLACK };

    draw.background().color(background);

    for depth in 0..=model.max_goals {
        let s = WIDTH.saturating_sub(10 * depth as u32);
        draw.rect()
            .no_fill()
            .stroke_weight(LINE_WIDTH)
            .stroke_color(foreground)
            .rotate(map_range(depth, 1, 30, 0.0, PI * 2f32))
            .w_h(s as f32, s as f32)
            .x_y(0.0, 0.0);
    }

    draw.to_frame(app, &frame).unwrap();
}
