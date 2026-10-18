# Viewer backend

The M8 gate (implementation plan §13.1): a small `wgpu` integration compared with what Xenoxide's
Vulkan renderer would take to draw into Lambda Slicer's window, on the same map, with the same
animated model, at the same sizes. This file records the measurements, the decision, and the
contract the viewer is built on from M9.

## Decision

**Keep `wgpu`, drawing offscreen on the window's own device and shown through egui's paint
callback.** The spike draws a lightmapped map and an animated player model at 0.3 ms a frame at
1920 by 1080, uses the device eframe already opened, and adds no second window, swapchain or GPU
copy. Xenoxide's renderer is a Vulkan client of its own inside the game's process. Making it draw
into this window would mean taking its swapchain, UI and device setup apart, which is far more work
than the spike and buys no speed the spike lacks. Its asset and pose code is already reused as
copies (docs/dependency-audit.md, "Viewer references").

In game on the same machine and map, Xenoxide runs at 400 to 650 fps (1.5 to 2.5 ms a frame),
which is its whole frame with its UI and the game's work. The spike's whole window frame, egui
included, is 1.2 to 2.7 ms. Neither number argues for switching.

## What the spike draws

`lambda-slicer-render`, with `lambda-slicer-assets` for reading:

- **World**: the world model's faces from the BSP, fanned into triangles, grouped by texture.
  Textures come from the map or from the WADs its `worldspawn` names, found through the game's
  search path. `{` textures are cut out at palette index 255. Style 0 lightmaps are packed into
  one 2048-wide atlas with repeated edge luxels. Sky faces are left out.
- **Model**: `models/player/gordon/gordon.mdl` with its sequence groups and texture companion,
  posed by the copied Xenoxide evaluator (`run`, looped from the clock, controllers and blends
  centred) and skinned on the CPU every frame, lit by one fixed direction.
- **Camera**: turning around the first deathmatch spawn, 140 units out and 40 up, the game's
  90 degree view at 4 by 3 (73.74 degrees high).

Left for M9 and later: mip levels, visibility (the whole world is drawn every frame), light
styles, the sky, brush entities, studio lighting and chrome, sprites and effects.

## Measurements

One machine, 2026-10-18: Windows 10 22H2, NVIDIA GeForce RTX 3070, release builds. wgpu picked
Vulkan both headless and in the window. Map `stalkx` (11,010 triangles, 51 textures, lightmap
2048 by 512) with the 362-triangle player model, `ag_bhop_champion_0_4_3` (11,867 triangles, 104
textures), the map Xenoxide was measured on, and `stalkyard` (6,421 triangles) for comparison.

### Procedure

- **Headless**, `cargo run --release -p lambda-slicer-render --example render_bench -- INSTALL ag
  stalkx 8 OUT.png`: 30 frames to warm up, then frames at 1920 by 1080 for 8 seconds. Every frame
  poses and skins the model, records, submits and waits until the GPU is done, so a frame's time
  is all of its work with nothing overlapped. Then 300 frames alternating 1280 by 720 and 1920 by
  1080 (a few pixels wider on some), remaking the targets every frame.
- **In the window**, `lambda-slicer-gui DEMO --size 1920x1080 --game INSTALL --viewer-bench 15
  OUT.txt`: vsync off, the demo view showing the viewer above its scores, clips and timeline. After
  a second to settle, the time between the window's frames for 15 seconds, egui and presenting
  included. The library scan and the demo's analysis run at the same time, as in normal use.
- **Memory**: bytes of every buffer and texture the scene makes (`Scene::gpu_bytes`), and the
  process's working set and private bytes 10 seconds into a run with and without `--game`.

### Results

| Measure | `stalkx` | `ag_bhop_champion_0_4_3` | `stalkyard` |
|---|---|---|---|
| Map read and meshed | 141 ms | 186 ms | 146 ms |
| Model read | 3.5 ms | 5.2 ms | 3.8 ms |
| Upload | 13 ms | 28 ms | 9 ms |
| Scene on the GPU, before targets | 8.6 MiB | 16.3 MiB | 4.6 MiB |
| Targets at 1920 by 1080 (colour and depth) | 15.8 MiB | 15.8 MiB | 15.8 MiB |
| Headless frame at 1920 by 1080, mean | 0.31 ms (3,255 fps) | 0.39 ms (2,564 fps) | 0.31 ms (3,210 fps) |
| Headless frame, p50, p95, p99 | 0.27, 0.50, 0.64 ms | 0.32, 0.68, 1.30 ms | 0.26, 0.55, 0.76 ms |
| Headless frame while resizing every frame, mean and max | 0.83, 1.44 ms | 1.10, 7.75 ms | 0.77, 1.44 ms |

| In the window (`stalkx`, view 1098 by 973 pixels in a 1920 by 1080 window) | Result |
|---|---|
| Window frame, mean over four runs, one while a build ran | 1.2 to 2.7 ms (829 to 365 fps), varying with the scan |
| Window frame, p99 | 2.3 to 4.7 ms |
| Scene on the GPU | 16.8 MiB |
| Working set and private bytes | 190 and 416 MiB with the viewer, 212 and 441 MiB without, the difference within the scan's noise |

The window's frame is mostly egui drawing the demo view. The scene's share is the headless
0.3 ms.

**Resizing** remakes the colour and depth targets in the frame the size changes and the blit's
bind group with them. It costs about half a millisecond, without a stall, and nothing is copied.

| Xenoxide in game (`ag_bhop_champion_0_4_3`, 1920 by 1080, its FPS counter) | Result |
|---|---|
| Frames per second while turning a full circle, lowest and highest | 400 and 650 (2.5 and 1.5 ms) |
| Early Xenoxide drawing the whole BSP with nothing else, on the AMD RX 580 used before, as its owner remembers it | about 1,500 |

Xenoxide was measured on `ag_bhop_champion_0_4_3`, its default map, because it starts a map
without an `info_player_start`, such as `stalkx`, at the map's origin. Its `find_viewer_start`
keeps the first deathmatch spawn as a fallback but returns nothing when the entity text ends.
Xenoxide starts at the `info_player_start` and the spike turns around a deathmatch spawn, which
matters little while the spike draws the whole map from anywhere.

Xenoxide presents with mailbox or immediate when the driver offers them, so its counter is not
held to the refresh rate. Its frame includes its own UI and the game's work, and its plan still
lists BSP visibility and lightmaps as to do, so it is a reference for the order of magnitude, not
a like-for-like figure. The early figure suggests most of today's frame is spent beyond drawing
the world, so Xenoxide's renderer would not make the viewer's world faster.

### Linux

The same headless bench under WSL 1 (no GPU, so llvmpipe's software Vulkan) builds and draws
`stalkx` correctly at 17.8 ms a frame at 1920 by 1080. That proves the code runs on Linux, not
how fast. CI builds and tests the crates on Linux. A Linux desktop with a GPU has not been tried.

## Integration cost

| | `wgpu` (kept) | Xenoxide's renderer |
|---|---|---|
| Device | the one eframe opened (`egui_wgpu::RenderState`) | opens its own Vulkan instance, device and swapchain (`xenoxide-render/src/vulkan.rs`) |
| egui | the app's 0.37, through the paint callback | pins egui 0.26.2 and `egui-ash-renderer` 0.1.0 for its own menus and HUD |
| Image handoff | a texture on the same device, sampled in egui's pass | would need an exported Vulkan image imported into wgpu, with semaphores, or a copy through the CPU |
| Synchronisation | none by hand, commands go in order on one queue | external semaphores between two devices, or one device shared through `wgpu`'s unsafe HAL access |
| Windows backend | whatever wgpu picks (Vulkan here, DX12 possible) | Vulkan only, so DX12 adapters would need the Vulkan backend forced |
| Linux | the same code, Vulkan or GL | its own surface code per platform (`surface_windows.rs`, `surface_linux.rs`) |
| Code brought in | the spike's four modules and the copied asset and pose code | `xenoxide-render` with `xenoxide-ui`, `xenoxide-game`, `xenoxide-map`, `xenoxide-platform`, `ash`, `naga`, `glam`, `fontdue` |
| Built for | the app's 64-bit process | a 32-bit `client.dll` loaded by Half-Life |

Its render graph and studio shading are worth reading when M9 ports lighting and materials, as the
plan says, but as algorithms brought over into `lambda-slicer-render`, not as a dependency.

## Contract

What the viewer keeps from here on:

- **One device.** `lambda-slicer-render` takes the device and queue it is given and never opens
  its own. Only the headless bench does, to measure.
- **Offscreen targets.** `gpu::Scene` owns an `Rgba8Unorm` colour target and a `Depth32Float`
  depth target at the view's size in pixels, remade only when the size changes.
- **Shown by a paint callback.** The GUI's `viewer.rs` draws the scene in
  `CallbackTrait::prepare`, whose command buffer egui submits before its own pass on the same
  queue, then `gpu::Blit` draws a triangle over the view's rectangle in `paint`. Nothing is read
  back to the CPU.
- **Colour in gamma space.** Textures and lightmaps stay as the game stores them, as the game blends
  them. The blit decodes to linear only when the window's format is sRGB.
- **Pinned together.** The render crate's `wgpu` is the one eframe uses (30.0.1 with egui 0.37),
  with default features off, so the backends are those eframe enables. They move together when
  eframe is updated.
- **CPU skinning for now.** One model skins in microseconds. GPU skinning waits until a scene with
  many players shows it is needed.
- **Playback drives poses.** The spike's looping clock is a placeholder. From 8d on, poses come
  from the playback controller's time and the recorded sequence, frame and blend values.

The demo view now plays the demo on this contract (docs/viewer-profile.md, "Viewer"), with
many posed model instances drawn from one vertex buffer a frame.
