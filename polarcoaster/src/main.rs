use polar_core::traces::{CoasterTrace, TraceEvent};
use raylib;
use raylib::prelude::*;
use std::fs::File;
use std::io::{prelude, Read};

fn main() {
    let filename = "coaster.json";
    println!("Loading {}", filename);
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let trace: CoasterTrace = serde_json::from_str(&contents).unwrap();

    let height = trace.max_depth;
    let mut width = 0;

    // need to add results so I can know which paths are successful

    // track how many points we have for each level.
    let mut num_in_level = vec![0; trace.max_depth+1];
    // try to figure out track paths by keeping track of where backtracking went to
    let mut last_in_level = vec![0; trace.max_depth+1];
    let mut current_level = 0;
    let mut edges = vec![];
    for (i, depthp) in trace.depths.iter().enumerate() {
        let depth = *depthp;
        num_in_level[depth] += 1;
        width = std::cmp::max(width, num_in_level[depth]);

        if i == 0 {
            assert_eq!(depth, 0);
            continue;
        }

        if depth == current_level {
            edges.push((last_in_level[depth], i));
            last_in_level[depth] = i;
        } else if depth > current_level {
            assert_eq!(depth, current_level+1);
            edges.push((last_in_level[current_level], i));
            last_in_level[depth] = i;
        } else if depth < current_level {
            edges.push((last_in_level[depth-1], i));
            last_in_level[depth] = i;
        }

        current_level = depth;
    }

    let mut node_positions = vec![Vector2::default(); trace.depths.len()];
    let mut seen_at_level = vec![0; trace.max_depth+1];
    for (i, depthp) in trace.depths.iter().enumerate() {
        let depth = *depthp;
        let offset = (width-1 - (num_in_level[depth]-1)) as f32 / 2.0;
        node_positions[i].x = seen_at_level[depth] as f32 + offset;
        node_positions[i].y = depth as f32;
        seen_at_level[depth] += 1;
    }

    // scale the thing to fit in the window. Not going to look good for really big or really
    // small traces but can handle that later.
    let window_width = 640 as f32;
    let window_height = 480 as f32;

    let width = (width-1) as f32;
    let height = (height) as f32;



    let scale = if width > height {
        window_width*0.8 / width
    } else {
        window_height*0.8 / height
    };
    let offset = Vector2{
        x: (window_width - (width * scale)) / 2.0,
        y: (window_height - (height * scale)) / 2.0
    };
    let scale = Vector2{
        x: scale,
        y: scale
    };

    // scale node positions
    for pos in &mut node_positions {
        *pos = *pos*scale+offset;
    }

    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();

    let mut cart_from = 0;
    let mut cart_to = 1;
    let mut cart_progress = 0.0;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        let p = Vector2::zero()*scale+offset;
        //d.draw_rectangle(p.x as i32, p.y as i32, (width*scale.x) as i32, (height*scale.y) as i32, Color::GRAY);

        for (fromp, top) in &edges {
            let from = *fromp;
            let to = *top;
            d.draw_line_ex(node_positions[from], node_positions[to], 2.0, Color::BLACK);
        }
        for pos in &node_positions {
            d.draw_circle_v(*pos, 3.0, Color::RED);
        }

        let cart_pos = node_positions[cart_from];
        let cart_size = Vector2{
            x: 10.0,
            y: 15.0
        };
        d.draw_rectangle_v(cart_pos-(cart_size/2.0), cart_size, Color::BLUE);



        //d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
        //d.draw_rectangle(center_x-half_cs*scale,center_y-half_cs*scale,coaster_size*scale,coaster_size*scale, Color::RED);
    }
}
