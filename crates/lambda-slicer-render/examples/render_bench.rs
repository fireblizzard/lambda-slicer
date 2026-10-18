//! Draws a map and a running player model offscreen and reports timings,
//! for the backend measurement in docs/viewer-backend.md
//!
//! `render_bench INSTALL GAME MAP [SECONDS] [OUT.png]`
//!
//! Every frame waits for the GPU, so a frame's time is the whole of its
//! work at 1920 by 1080. Afterwards the size changes every frame for a
//! while, to time remaking the targets, and one frame is saved when `OUT`
//! is given

use std::time::{Duration, Instant};

use lambda_slicer_assets::Limits;
use lambda_slicer_assets::search::{SearchOptions, SearchPath};
use lambda_slicer_render::camera::Camera;
use lambda_slicer_render::gpu::{Brush, COLOR_FORMAT, Instance, Scene, View};
use lambda_slicer_render::{load, world};

const SIZE: [u32; 2] = [1920, 1080];

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn report(label: &str, times: &mut [f64]) {
    times.sort_by(f64::total_cmp);
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    println!(
        "{label}: {} frames, mean {mean:.3} ms ({:.0} fps), p50 {:.3}, p95 {:.3}, p99 {:.3}, max {:.3} ms",
        times.len(),
        1000. / mean,
        percentile(times, 0.5),
        percentile(times, 0.95),
        percentile(times, 0.99),
        times[times.len() - 1],
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [install, game, map, ..] = args.as_slice() else {
        eprintln!("usage: render_bench INSTALL GAME MAP [SECONDS] [OUT.png]");
        std::process::exit(2);
    };
    let seconds: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(10.);
    let out = args.get(4);

    let started = Instant::now();
    let search = SearchPath::new(
        install.as_ref(),
        game,
        &SearchOptions::default(),
        Limits::default(),
    )
    .expect("search path");
    let (mesh, unread) = load::map(&search, map, Limits::default()).expect("map");
    let map_ms = started.elapsed().as_secs_f64() * 1000.;
    let started = Instant::now();
    let player = load::player(&search, "gordon").expect("player model");
    let model_ms = started.elapsed().as_secs_f64() * 1000.;
    println!(
        "map {map}: {} triangles in {} batches, {} textures ({} missing), lightmap {}x{}, WADs not read {unread:?}, {map_ms:.1} ms",
        mesh.indices.len() / 3,
        mesh.models.first().map_or(0, Vec::len),
        mesh.textures.len(),
        mesh.missing_textures.len(),
        mesh.lightmap.width,
        mesh.lightmap.height,
    );
    println!(
        "model {}: {} triangles, {} sequences, {model_ms:.1} ms",
        player.path,
        player.mesh.corner_count() / 3,
        player.model.sequences().len(),
    );

    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .expect("adapter");
    let info = adapter.get_info();
    println!("adapter: {} ({:?})", info.name, info.backend);
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Default::default())).expect("device");

    let started = Instant::now();
    let mut scene = Scene::new(&device);
    scene.set_world(&device, &queue, &mesh);
    let model = scene.add_model(&device, &queue, &player.mesh);
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("upload");
    println!(
        "upload: {:.1} ms, {:.1} MiB on the GPU before the targets",
        started.elapsed().as_secs_f64() * 1000.,
        scene.gpu_bytes() as f64 / 1048576.
    );

    let (spawn, yaw) = world::spawn(&mesh.entities);
    let sequence = player.sequence("run").unwrap_or(0);
    let frame = |scene: &mut Scene, t: f64, size: [u32; 2]| {
        let angle = t as f32 * 0.4;
        let eye = [
            spawn[0] + angle.cos() * 140.,
            spawn[1] + angle.sin() * 140.,
            spawn[2] + 40.,
        ];
        let camera = Camera::looking_at(eye, spawn, 73.74);
        let bones = player.bones_at(sequence, t, spawn, yaw).unwrap_or_default();
        let instance = Instance {
            model,
            vertices: player.mesh.skin(&bones),
        };
        let view_projection = camera.view_projection(size[0] as f32 / size[1] as f32, 4., 8192.);
        let view = View {
            view_projection,
            eye,
            time: t as f32,
            light_styles: [1.; 64],
        };
        let (_, commands) =
            scene.render(&device, &queue, size, &view, &[Brush::world()], &[instance]);
        queue.submit([commands]);
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("frame");
    };

    // The first frames make the targets and warm the driver
    for i in 0..30 {
        frame(&mut scene, f64::from(i) / 60., SIZE);
    }
    let mut times = Vec::new();
    let begin = Instant::now();
    while begin.elapsed() < Duration::from_secs_f64(seconds) {
        let at = Instant::now();
        frame(&mut scene, begin.elapsed().as_secs_f64(), SIZE);
        times.push(at.elapsed().as_secs_f64() * 1000.);
    }
    report("1920x1080", &mut times);
    println!(
        "GPU memory held: {:.1} MiB",
        scene.gpu_bytes() as f64 / 1048576.
    );

    let mut resizing = Vec::new();
    for i in 0..300u32 {
        let size = [1280 + (i % 2) * 640 + i % 7, 720 + (i % 2) * 360];
        let at = Instant::now();
        frame(&mut scene, f64::from(i) / 60., size);
        resizing.push(at.elapsed().as_secs_f64() * 1000.);
    }
    report("resizing every frame", &mut resizing);

    if let Some(out) = out {
        frame(&mut scene, 1.25, SIZE);
        save(&device, &queue, &scene, out);
        println!("saved {out}");
    }
}

/// The colour target written to a PNG
fn save(device: &wgpu::Device, queue: &wgpu::Queue, scene: &Scene, out: &str) {
    let view = scene.color_view().expect("a frame was drawn");
    let texture = view.texture();
    let row = SIZE[0] * 4;
    let padded = row.div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(padded * SIZE[1]),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: None,
            },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);
    buffer.map_async(wgpu::MapMode::Read, .., |r| r.expect("map"));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("readback");
    assert_eq!(texture.format(), COLOR_FORMAT);
    let data = buffer.get_mapped_range(..).expect("mapped");
    let pixels: Vec<u8> = data
        .chunks(padded as usize)
        .flat_map(|r| &r[..row as usize])
        .copied()
        .collect();
    image::save_buffer(
        out,
        &pixels,
        SIZE[0],
        SIZE[1],
        image::ExtendedColorType::Rgba8,
    )
    .expect("png");
}
