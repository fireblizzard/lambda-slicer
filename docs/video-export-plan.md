# Video export implementation plan

Status: consolidated design, ready for implementation. First deliverable: a verified 10-second real-demo MP4 with sound (V0).

Prepared 2026-10-23 for TODO item 57, “Add the ability to export to different video formats that can be uploaded directly to platforms like YouTube or TikTok.” Based on repository commit `20b7340` and the working TODO at that date. Consolidated 2026-10-23 after the design reviews. Existing uncommitted application work is outside this documentation change. Paths below are repository-relative. Recheck named interfaces before implementing, since viewer work is continuing.

## 1. Outcome and decisions

Export an existing marked clip, several clips as separate files, or an entire demo to a normal video with synchronized game sound. The video contains the rendered game and chosen HUD, not the Lambda Slicer window. Export must work while the app is minimized and without an audio device. Rendering may run slower or faster than real time without changing the video's speed or dropping frames.

Recommended implementation:

- Render offline using Lambda Slicer's existing Rust playback and wgpu renderer. Use the user's installed assets through the existing search rules; do not launch `hl.exe` or distribute game assets.
- Use FFmpeg as a separate executable for encoding/muxing, and ffprobe for verification. Do not initially link libavcodec/libavformat into Rust.
- First finish an MP4 path using H.264 video and AAC-LC stereo audio. Add size targets and plain-cut joining next, then WebM/VP9/Opus as a follow-up format. Keep one tested editing preset, MOV/DNxHR/PCM, as a later milestone. MP4 alone is a useful first release but does not complete the full multi-format TODO.
- Offer YouTube Landscape (1920×1080, 60 fps) and TikTok / Shorts Portrait (1080×1920, 60 fps), plus 30 fps and custom settings. These are application presets, not guarantees about platform/account upload eligibility.
- Use software `libx264` as the reference encoder. Add hardware acceleration after correctness and cancellation are proven.
- Keep the export runner independent of its thread/process host. The first experiment may use a background thread with private state; the production runner uses a supervised private worker process. Stream unclipped stereo float audio to a temporary file, measure its true peak, then encode video with one fixed audio gain. This avoids two simultaneously writable media pipes and their cross-platform deadlocks.
- Preserve the viewer's display RGB appearance, interpolate recorded camera poses without advancing future effects, and render portrait crops at their final pixel density.
- Reuse the existing `.dem` export's source-snapshot and no-clobber publication guarantees from the first tester build, adapting its writer interface for FFmpeg.
- Freeze each job's input range, settings and dependencies when queued. Do not use the interactive viewer's live clock, mute switch, loop or selected demo as export state.

The initial release includes 1× forward playback, recorder/spectator POV as recorded, landscape/portrait composition, selectable HUD, audio, cancellation, a serial queue, whole-demo export and precise error reporting. A following milestone adds ordered joining with plain cuts, not a montage editor. Defer transitions, automatic action tracking, free-camera animation, speed ramps, reverse video, pitch-preserving slow motion, motion blur, HDR, direct platform upload and crash-resume. Do not silently inherit any of those behaviors from the live viewer.

## 2. What already exists, and what must change

| Existing location | Reuse | Required work / trap |
|---|---|---|
| `crates/lambda-slicer-playback/src/source.rs` | Streaming groups in file order and stable group numbers | Source timestamps alone do not uniquely identify a position across sessions. |
| `crates/lambda-slicer-playback/src/playback.rs` | `Playback`, `set_game_files`, checkpoints, `seek_group`, `advance_to`, `present`, `PresentedFrame` | Add a bounded, group-aware offline advancement interface. Inspect `PresentedFrame.stopped`; `advance_to` returning `Ok` alone does not establish complete playback. |
| `crates/lambda-slicer-playback/src/time.rs` | Separate source/server clocks and `Position { group, segment, time }` | Add an output-to-source mapping and anchored camera track; never use server time or wall time as output timestamps. |
| `crates/lambda-slicer-gui/src/viewer.rs` | Asset loading, readiness, draw-list building, posing, camera setup, HUD composition | Extract reusable presentation functions. Current `Capture`/`step_capture` is a screenshot diagnostic, not a video exporter. |
| `crates/lambda-slicer-render/src/gpu.rs` | Offscreen `Scene::render`, `color_view`, `read_color` | `read_color` allocates and waits indefinitely per capture. Start with a bounded-wait version, then add reusable readback buffers. |
| GUI `kill_feed.rs`, `chat.rs`, `announcements.rs`, `vitals.rs`, `mini_scores.rs`, `hud_messages.rs`, `crosshair.rs`, `weapon_menu.rs`, `hud_sprites.rs` | Current overlay drawing and sprite/font handling | HUD is drawn through egui after the scene callback. Reading the scene texture alone omits it. |
| `crates/lambda-slicer-audio/src/library.rs` | Sound assets, search paths and `Library::game_files()` callbacks | Use the same asset resolution for picture and sound, including session changes. |
| `crates/lambda-slicer-playback/src/audio.rs` and audio `mixer.rs` | Active-channel rules, `Listener`, `follow`, pitch/loop cursors | Keep active voices separate from bounded mixing history; add timed volume/end handling and unclipped float output. Current all-samples `write_wav` is inappropriate for long exports. |
| `crates/lambda-slicer-audio/examples/mix_probe.rs` | Reference for offline sound generation | Do not copy its accumulating `Vec` or assume end-of-block channels describe everything heard during that block. |
| `crates/lambda-slicer-gui/src/clips.rs`, `demo/clips_tab.rs`, `export.rs` | Requested ranges, committing edits, stable runtime clip IDs, existing queue conventions | Keep `.dem` export separate. Video must not first make a sliced demo or inherit first-session-only slicing restrictions. |
| `crates/lambda-slicer-core/src/slice/output.rs` | Source snapshot/hash, sync/validation and `persist_noclobber` publication | Extract a reusable path-producer/finalization boundary for FFmpeg; the existing callback supplies an open Rust file, not an output pathname. |
| GUI `settings.rs`, `app/settings_window.rs`, locales | Persistence and localized controls | Add video settings without changing viewer settings or invalidating existing clip JSON. |
| `scripts/check.ps1`, `scripts/package.ps1`, `.github/workflows/release.yml` | Checks and packaging | Add exporter checks and document FFmpeg discovery; bundling is a separate packaging decision. |

The exported picture/sound initially has the same supported features and limitations as the native viewer. Review `docs/viewer-profile.md`, `docs/viewer-validation.md`, and `docs/limitations.md` at implementation time (the concurrent documentation cleanup replaces `docs/deferred.md`). Export is not proof of Half-Life rendering/audio parity. Track missing sentences, effects or other unsupported behavior as actual current limitations, rather than copying stale lists from this plan.

## 3. User workflow

### 3.1 Entry points

1. Add **Export video…** beside the existing demo/clip export actions. Keep **Export demo… (.dem)** clearly separate; never change what an established Export demo button writes.
2. A clip card exports that clip. The Clips toolbar exports selected clips, or all clips if the user explicitly chooses that scope. Each clip becomes a separate video by default.
3. The open demo's menu and library row context menu offer **Export entire demo as video…**. This must not require creating a permanent clip.
4. Commit active clip name/time edits before capturing a job. Invalid edits must prevent enqueueing and explain which field is invalid.
5. Open one export window with a real preview, range summary, output settings and destination. Show missing game-file or encoder setup inline with Locate/Browse actions.

Main controls: scope, preset, landscape/portrait layout, quality or target file size, 30/60 fps, output folder, sound on/off, HUD on/off, preview, Export. Advanced controls: framing/FOV policy, individual HUD groups and HUD scale, crop position, output size, encoder, output gain, overlay language, privacy, background mode and diagnostics. Hide raw codec terminology from the main workflow except the file type; label MP4 as the recommended compatible choice.

Provide both representative stills (start, middle, end, selected event) and a short playable preview through the same compositor and sound path. Start with a 3–5 second preview around the selected event; disclose a reduced preview resolution while preserving framing, HUD scale and timing. Cancel stale previews when settings change. It does not seek or pause the interactive viewer. Display the exact crop and optional platform-safe-area guides. Guides are adjustable design aids, not permanent claims about platform overlays, and are never burned into output.

Show measured **camera update rate** when useful: derive it from valid camera-sample intervals within sessions, excluding resets and duplicate times, and report sparse/irregular stretches. Do not call it network tick rate or the recorder's monitor refresh rate. A 60 fps export can have smooth interpolated camera motion, but does not recover unrecorded input detail. Below 60 camera samples/s, unsmoothed camera poses may repeat while other animations continue; do not claim entire frames necessarily repeat.

Offer editable **Suggested match range** when analysis identifies a reliable match start/end. Show its evidence and bounds; keep **Entire demo** complete by default. Automatic removal of pauses/timeouts is deferred to an explicit edit-list feature. Do not silently trim warmup, intermission or gaps.

### 3.2 Queue and completion

Use states `Queued → Checking → Preparing audio → Rendering/encoding → Finalizing → Verifying → Completed`, with `Cancelling`, `Cancelled`, `Failed`, and `Completed with warnings`. Progress names the phase, clip, completed/total frames or samples, and elapsed time. ETA appears only after sufficient measurements; finalization can continue after all frames have been rendered.

One GPU export at a time. Users can keep browsing, switch demos, edit future clips, or cancel an active job. Queued jobs retain their captured settings and clip ID; edits do not mutate a running export. Default to prioritizing interactive playback by reducing export throughput and limiting encoder CPU threads; slowing export must not drop frames. Offer an explicit full-speed option. On app close, offer cancel jobs and close or keep working. Child processes must not be orphaned.

Completion provides Open video, Show in folder, and View report. Encoding does not upload media or make network requests. A user-initiated FFmpeg installation is a separate, disclosed download action. Errors retain a readable cause and a bounded diagnostic log. Do not report success merely because FFmpeg created a file.

### 3.3 Defaults and saved settings

Persist a versioned `VideoExportSettings` object with tolerant defaults: last preset, format, folder, crop/layout choices, encoder path and quality. Copy current picture/HUD settings into the export dialog on first use, show them, then freeze the selected values into the job. Export gain defaults to 100% of the unclipped game mix, independently of preview mute/system volume, with peak protection by a measured reduction-only fixed gain (§8). Show the resulting gain adjustment in the report. HDR and alpha output are not supported initially. See-through walls is off by default and requires an explicit export choice with a visible summary label.

Overlay language defaults to the application language, is editable before export, and is frozen with the job. Do not translate recorded player chat. Privacy settings must cover chat, player labels, kill feed, scoreboard, HUD messages and any generated thumbnail, subtitles, chapters, title/description or file metadata. Use stable pseudonyms across a joined video when anonymizing; suppress free-form text that cannot safely be anonymized. Apply privacy before composition, including blurred backgrounds and previews.

Preserve existing clip files. Any later addition of session/group anchors needs a backward-compatible optional-field migration. Legacy clips resolve to their existing first-session semantics; do not guess a later occurrence of the same timestamp.

## 4. Formats, presets and quality

Container, video codec and audio codec are different choices. MP4 is a container; H.264 encodes picture; AAC encodes sound. Changing a suffix does not convert a format.

| Preset / format | Initial settings | Purpose |
|---|---|---|
| YouTube Landscape | MP4, H.264 High, AAC-LC stereo, 1920×1080, 60 fps, 48 kHz | Default full match / landscape clip. |
| TikTok / Shorts Portrait | MP4, same codecs, 1080×1920, 60 fps, 48 kHz | Portrait composition with fit/crop preview. |
| Compact quality | MP4, 1280×720 landscape or 720×1280 portrait, 30 fps | Usually smaller at comparable settings; CRF does not guarantee size. |
| Target file size | MP4, editable byte budget, bitrate-constrained encoding | Sharing limits; verify actual bytes and offer explicit quality/resolution tradeoffs (§9.4). |
| WebM, follow-up | VP9 + Opus, same layout/resolution options | Second delivery format after the usable MP4 path and sharing workflow. |
| Editing master, later | MOV, DNxHR HQ, 48 kHz stereo PCM (start with 24-bit) | First editing-oriented preset; test import, seeking, audio and portrait dimensions in the actual editor. |

YouTube's current recommendations include MP4/H.264, progressive 4:2:0, BT.709 SDR, 48 kHz audio and fast-start metadata. Its 1080p60 reference bitrate is 12 Mb/s; that is a reference, not a required constant bitrate or quality ceiling. The app's default 60 fps is a rendering choice for a demo, which is not an existing video with an intrinsic encoded frame rate. [YouTube encoding guidance](https://support.google.com/youtube/answer/1722171?hl=en).

TikTok's Content Posting API documentation recommends MP4/H.264 and lists 23–60 fps, dimensions of 360–4096 pixels per side and a 4 GB ceiling. Those are **API-path constraints**, not a complete description of manual website/app uploads or every account's duration allowance. Use 1080×1920 at 30/60 fps as our conservative preset; advise users to check their account's duration limit instead of hard-coding the API's limit into a local exporter. Allow a whole 20-minute demo to be exported even if a platform later requires trimming. [TikTok media transfer guide](https://developers.tiktok.com/docs/en/content-posting-api-media-transfer-guide).

Software H.264 starting choices: High quality CRF 18, Balanced CRF 20, Compact CRF 23, `preset medium`. These are project tuning defaults, not platform requirements; compare fast camera motion, thin railings, blood/sparks, dark rooms, gradients and colored HUD text before finalizing. Do not offer CRF 0/RGB H.264 as the normal upload preset. Explain that CRF controls quality and produces a variable file size. A size estimate is approximate unless using an explicit bitrate budget. Start AAC at 384 kb/s; WebM Opus at 192 kb/s. For VP9 begin with constant-quality CRF 30 and zero target bitrate, then tune on the same corpus; never transfer x264 quality numbers directly to another codec.

Add 1440p/4K only after memory/throughput tests, using supported even dimensions and matching hardware limits. Do not pretend 4K creates missing source detail. HEVC and AV1 are later alternatives: potentially smaller files, but different encoder availability, speed and playback compatibility. Keep H.264 as the portable default. [FFmpeg codec options](https://ffmpeg.org/ffmpeg-codecs.html#libx264_002c-libx264rgb).

### 4.1 Platform advice and editor compatibility

Keep platform restrictions as dated advice, not universal export prohibitions. As checked 2026-10-23, YouTube Shorts supports square/vertical videos up to three minutes; longer portrait exports remain valid files. Discord's FAQ states a 20 MB free upload limit and notes experiments; allow an editable limit and leave safety margin. Its developer documentation expresses the default as 20 MiB, so store exact bytes internally and label units; a conservative 20,000,000-byte preset fits either interpretation. X's standard video guidance lists 140 seconds; recheck current account-specific rules before release. Sources: [YouTube Shorts](https://support.google.com/youtube/answer/15424877?hl=en), [Discord FAQ](https://support.discord.com/hc/en-us/articles/25444343291031-File-Attachments-FAQ), [Discord upload reference](https://github.com/discord/discord-api-docs/blob/main/developers/reference.mdx), [X video guidance](https://help.x.com/en/using-x/x-videos).

The normal H.264/AAC MP4 is a delivery preset, not a promise of compatibility with every editor. Blackmagic's July 2025 **Resolve 20** table lists H.264 decoding on Linux as Studio-only and AAC as unsupported on Linux even without a Studio exception. It lists DNxHR/ProRes decoding and PCM support. Document that versioned limitation in the user guide, and verify the actual target editor/version before advertising the editing preset. [Resolve 20 codec table, pages 11 and 14](https://documents.blackmagicdesign.com/SupportNotes/DaVinci_Resolve_20_Supported_Codec_List.pdf).

High-quality H.264 remains useful for compact compilations. All-intra H.264 is a later benchmark, not a supported preset: independent frames alone guarantee neither ProRes-like scrubbing nor smaller files at comparable quality. ProRes and lossless FFV1 are later alternatives if a measured workflow calls for them; do not add a codec matrix to the initial UI. DNxHR output should come from rendered RGB, not a previously compressed MP4. Increased bit depth does not restore absent source precision.

## 5. Architecture and module boundaries

Add `crates/lambda-slicer-video` for job types, time scheduling, offline audio orchestration, FFmpeg process management, verification and reporting. It may depend on core, playback, assets and audio **without** the `device` feature. Keep it independent of eframe, live UI state and a sound device. The rendering boundary accepts a `PresentedFrame` and frozen composition settings and produces an identified RGBA frame; use a trait so timing and encoder tests can use generated frames without game assets/GPU.

Extract `presentation/` modules inside the GUI crate from `viewer.rs`: asset/readiness handling, scene assembly/posing, camera/view calculation and a HUD renderer. Both the interactive viewer and headless video worker call these modules. This avoids duplicating the rapidly evolving scene/HUD logic without requiring a broad crate reorganization. An independent shared presentation crate can follow later if useful; do not introduce a GUI↔video dependency cycle.

Keep the runner callable with frozen inputs, progress events and a cancellation token independently of hosting. V0 may use a background thread with private playback, sound and offscreen state, without a new process/IPC framework. Extract only presentation functions needed to render real footage; avoid duplicating renderer logic. Production uses process supervision before general release: it contains ordinary worker crashes and allows bounded termination of stuck code. It cannot guarantee protection from a graphics-driver reset affecting other processes.

Add GUI modules `video_export.rs` (dialog/queue) and `video_worker.rs` (headless runner). Dispatch a private `--video-worker <job-file>` mode in `main.rs` **before** creating the eframe app, reading user preferences, starting scans or creating an audio device. Use `current_exe()` and an absolute path to launch it. It creates its own wgpu device and isolated egui context. The same mode can provide a developer CLI driven by a versioned job specification; the existing analysis CLI need not acquire a GUI dependency.

Suggested interfaces, to adapt to actual Rust types:

```rust
struct ExportPosition { group: u64, segment: u32, source_us: i64 }
struct Rational { num: u32, den: u32 }
struct VideoJob {
    schema: u32,
    id: JobId,
    source: SourceIdentity,
    edit: ExportEditList, // one anchored range initially; ordered cuts later
    fps: Rational,
    width: u32,
    height: u32,
    composition: CompositionSettings, // source framing, canvas, placement, HUD
    privacy: PrivacySettings,
    picture: FrozenPictureSettings,
    audio: AudioSettings,
    encoding: EncodingSettings,
    destination: PathBuf,
}
trait FrameRenderer {
    fn prepare(&mut self, frame: &PresentedFrame) -> Result<AssetReadiness>;
    fn render(&mut self, index: u64, frame: &PresentedFrame,
              settings: &CompositionSettings) -> Result<VideoFrame>;
}
struct VideoFrame { index: u64, width: u32, height: u32, rgba: Vec<u8> }
```

These are proposed interfaces, not existing API signatures. `prepare` must have a terminal failure state, not spin forever. Frame identity includes the job generation and output index. If asynchronous GPU completions arrive out of order, reorder within a bounded window before writing. No frame may be duplicated, omitted or accepted from another job.

Capture source identity/fingerprint when queueing and verify it when creating the snapshot; if it changed, fail with a requeue action instead of silently using new content. Job metadata includes immutable source snapshot identity, resolved ranges and anchors, installation/search options, application version, asset manifest, locale/font settings, renderer/color-pipeline version and selected encoder/build. Snapshot each distinct source once per job and use it for preflight, audio, video and any retry; preserve its original name only as optional, privacy-filtered metadata. Keep native paths lossless in IPC: JSON path strings alone cannot represent arbitrary Linux filenames. Reuse the project's native-path encoding or define platform-tagged native bytes. Do not execute commands recorded inside demos.

## 6. The export clock and exact boundaries

### 6.1 Resolve the range before rendering

Reuse requested clip `[start, end)` values, not the `.dem` slice engine's snapped export range. Video can begin between recorded groups by presenting reconstructed state at the chosen time. Do not add pre/post-roll again to an already padded saved clip. The dialog displays requested duration and effective output duration after frame quantization.

An entire demo includes all supported playback sessions in file order, not just the session with the most kills, the longest match, the first map or the library's displayed duration. LOADING state is preparation and is not automatically turned into an opening black video interval. Decode all initialization required for the first visible state.

Build a compact `ExportTimeline` during preflight from record-group ordering and decoder session changes. Each entry stores the group, segment, source timestamp and mapped monotonic export time. Do not sort groups by timestamp. Repeated timestamps in one session preserve record order; all groups belonging to an output instant must be applied in order. Server timestamps are never used as seek identifiers.

Source-time runs that are monotonic retain their elapsed durations. A session boundary with a continuing source clock retains the recorded gap. A source clock reset creates a new anchored run at the preceding run's end; do not invent a negative interval or replay the earlier session. Use a valid recorded terminal boundary when available. If the duration of the final state before a reset/EOF is absent, use one output-frame interval as an explicit fallback, record it in the report, and expose the resulting duration before export. Invalid/nonfinite times or unexplained backward jumps inside a session fail preflight with the offending group until a tested recovery rule exists. Never silently clamp all decreases to zero.

Represent resolved endpoints with group/session anchors plus subframe offsets. Add a `Playback` advancement method constrained to the relevant group range/run and source target. Calling `advance_to(SourceTime(5.0))` alone cannot safely select the second occurrence of 5 seconds. At a discontinuity, process the transition in record order and reset session-local simulation/HUD/audio according to decoded events; do not reset unrelated state just because a directory entry changes.

Existing clip persistence only stores first-session times. Preserve that behavior for legacy clips. For video ranges in later sessions, resolve explicit anchors from the indexed whole-demo preview and store them in the job; adding persistent later-session clips is an optional compatible extension. Never relabel a later-session range as an exportable `.dem` slice unless the separate slicing engine supports it.

### 6.2 Integer frame and sample scheduling

For requested output duration `D_us > 0`, fps `p/q`, and audio rate `R = 48000`:

```text
N = ceil(D_us * p / (1_000_000 * q))
video timestamp(i) = i * q / p,  i = 0 .. N-1
effective duration T = N * q / p
S = round(T * R) stereo sample frames
sample timestamp(j) = j / R,     j = 0 .. S-1
```

Use checked integer/rational arithmetic (u128 intermediates), converting to f64 only at existing playback/render API boundaries. Do not accumulate `time += 1/fps` or base scheduling on elapsed wall time. Support integer 30/60 first; keep the rational type so 30000/1001 and 60000/1001 can be tested before exposing them.

At 1×, map each output timestamp to the resolved source interval. Apply groups through that exact presentation time, bounded by the run; include events at the start and exclude events at the requested end. Video timestamps before `D` use actual reconstructed states. The final video frame may persist for less than one extra frame beyond the requested duration; fill the matching audio tail from `D` to `T` with silence, not events beyond the selected range. Never append arbitrary gameplay to fill an encoder packet. Sub-frame clips still produce one frame; show this quantization.

Pre-roll is state reconstruction: decode from initialization or a compatible complete checkpoint, retaining active loops, temp entities, decals, animation, HUD and pending events. A fixed “decode two seconds before the clip” is insufficient. Do not output those warm-up frames/samples. Bind checkpoint reuse to the source, assets and simulation settings; initially use private export checkpoints to avoid interference from live playback.

Recorded pauses/intermissions are preserved in their source timing. Host rendering/loading stalls contribute zero output time. Test a multi-map recording, a clock reset, a mid-event clip start and a clipped existing `.dem` separately.

Define one `map_output_time` operation returning the edit-list span, source identity, session/group anchor and local source offset. Initially each span runs forward at 1×; frame and audio scheduling call this same mapping. Reserve rational rates in the model, but reject unsupported rates in runnable jobs. This avoids baking 1× into every caller without claiming slow motion is trivial: future work must define audio pitch/time-stretch, sample resampling, effect clocks, subtitle/chapter retiming and motion-blur exposure. Output may use 30000/1001 or 60000/1001 after validation when matching external footage; do not reject those rates as inherently unsuitable for games.

### 6.3 Shared camera sampling and discontinuities

Build a camera track alongside `ExportTimeline` during preflight, using existing protocol decoding. Each sample stores group/record ordering, segment, source time, camera origin/angles, recorded FOV, viewed entity, observer mode/target, available lifecycle state and a break reason before that sample. Preserve equal-time record order; use the final applicable sample at a presentation instant, never divide by zero or sort samples across sessions. Obtain FOV and observer identity from decoded state, not only raw position/angle fields.

Expose sampling in playback so the live viewer and exporter share the same interpolation. Between valid samples in a continuous interval, use linear position interpolation and shortest-path wrapped angle interpolation. Hold FOV until its next recorded change initially; preserve every recorded change rather than assuming all mods use instantaneous zoom. Keep camera position, viewmodel orientation and the audio listener consistent with this sampling contract. Do not add bob/punch/recoil already present in recorded poses a second time. Allow recorded stepping as an explicit alternative; do not extrapolate across EOF, malformed data or unknown gaps.

Prefer explicit break evidence: session/map changes, view-entity changes, spectator mode/target changes, supported respawn/death transitions and imposed view-angle resets. Stock Half-Life teleport and spawn handling set `fixangle`; ordinary `fixangle` is sent as `svc_setangle` in ReHLDS. This is evidence of an imposed angle change, not a universal teleport marker, and belongs to the receiving client. A watched player's teleport may not send it to the recorder. Sources: [HLSDK teleports](https://github.com/ValveSoftware/halflife/blob/master/dlls/triggers.cpp), [HLSDK spawn placement](https://github.com/ValveSoftware/halflife/blob/master/dlls/gamerules.cpp), [ReHLDS client messages](https://github.com/rehlds/ReHLDS/blob/master/rehlds/engine/sv_main.cpp).

Validate which camera sample first reflects each reset rather than attaching the break blindly to message arrival. Retain a conservative movement/gap fallback using displacement, elapsed time and available velocity/state; a fixed distance cutoff can misclassify legitimate fast AG movement. Do not globally reset effects or audio merely because camera interpolation is broken. On a discontinuity or uncertainty, hold the earlier pose until the next sample and snap then. This can last several output frames. Record the reason in diagnostics; prefer a documented conservative hold over interpolating through a wall.

The live viewer must not wait for a full scan. Use bounded lookahead to the next **valid** camera sample, potentially through several records/groups, with separate decoder state and existing decoding logic. Read its required FOV/observer/break metadata without publishing future kills, sounds, HUD or simulation changes. Bound work by records/bytes and source span; cancellation interrupts it. If no safe pair is found within the budget, hold the current pose. Clear stale lookahead on seek/load and test that it does not alter live state. Do not create a second partial protocol parser.

At 100 samples/s a 20-minute demo has about 120,000 camera samples. Use compact storage and an explicit memory budget; chunk/spill the preflight track and index for unusually long/high-rate demos, rather than allowing unbounded resident growth. Cache identity includes source and decoding policy. Shared sampling, not identical full-track storage, is required between live and offline paths.

## 7. Scene, HUD, portrait and color

### 7.1 Offscreen composition

For each scheduled picture: obtain `PresentedFrame`, finish required asset loading, build the same scene draw list as the viewer, render to a fixed-size offscreen target, composite the selected HUD, then read back the final composed texture. An empty model placeholder during a background load is not an acceptable completed frame. First use can wait; waits must remain cancellable. Reuse uploaded assets across frames with bounded cache eviction.

Create a headless egui context with fixed output resolution, deterministic pixels-per-point, fixed fonts/locale and time driven by output/source time as appropriate. Run only extracted game-overlay functions, tessellate their output, upload all texture deltas, and render with egui-wgpu into the final target after the scene. Process texture frees after their last GPU use. No window panels, toolbar, hover states, cursor, diagnostics or export progress are captured. Avoid calling the interactive `viewport_ui` with synthetic mouse events.

HUD groups: crosshair; health/armor/ammo/pickups/damage/weapon menu; kill feed; mini-scoreboard; chat; announcements/server settings; HUD text; watched-player label. Default from the visible viewer settings but permit overrides. Hide spectator-inappropriate HUD using the same rules as live viewing. Tests must include Unicode names, AG colors, right-to-left text, sprite fonts and an overlay active at the clip's first frame. Audio should not become inaudible merely because `present(... audible: false)` is used for offline image presentation; exporter audio is independently scheduled.

### 7.2 Framing, canvas and placement

Model three separate choices: **source framing** (reference aspect/FOV), **output canvas** (dimensions/aspect), and **placement** (fit/crop, with explicit stretch only in advanced compatibility settings). An output width/height must not implicitly redefine all three. Start with 16:9 and 9:16 canvases, then add 1:1 and 4:5 through the same composition model after validation.

Expose two descriptive FOV policies: **Keep horizontal field of view** (current viewer behavior), and **Preserve a 4:3 reference view's height**. With reference horizontal FOV 90°, preserving 4:3 vertical FOV yields about 106.26° horizontally at 16:9. Compute vertical FOV from the chosen reference aspect, then project onto the placement rectangle. Keep the current policy as the default when copying viewer settings; never silently change existing framing. Apply the policy to recorded FOV updates and explicit overrides consistently. Valve confirms a widescreen correction option in its anniversary update, but do not label a formula as matching the user's engine/configuration without a comparison. The demo does not establish the recorder's physical monitor or stretched display setup. [Valve release notes](https://www.half-life.com/en/halflife/25th).

Allow a 4:3 reference view fitted into 16:9 with side bars. A separately labeled **Stretch 4:3 to canvas** may emulate a user-selected stretched presentation; show the distortion in preview and never enable it implicitly. It must remain distinct from widening the FOV and from the portrait preset.

### 7.3 Portrait rendering and HUD layout

Ship **Fit gameplay** as the portrait default: place the complete chosen landscape reference view in the portrait canvas using uniform scale, with solid black or an optional blurred scene-only background. At 1080 pixels wide, a 16:9 picture is about 607.5 pixels high; choose even raster dimensions and document subpixel/rounding handling without stretching geometry. Fit preserves side action but makes the gameplay area smaller. The preview must show this tradeoff.

Provide a portrait HUD layout that uses canvas width and a dedicated UI scale, not the full 1920-pixel height passed into the existing landscape height-scaling code. Start from a 1080-wide logical canvas; scale with output width plus an explicit user multiplier. Lay out score/kill feed in the upper spare area and vitals/ammo/watched-player label below the gameplay area; keep the crosshair tied to the projected aim point. Resolve chat/announcements within defined safe rectangles. Clamp measured text and sprite layouts, account for long/RTL names, and verify on a phone-sized preview. For landscape use the established gameplay HUD layout. Reuse overlay components with explicit rectangles/scales rather than feeding them a fake giant viewport. An optional zoom inset is later work, not a prerequisite.

Offer **Fill by cropping** explicitly, with centered crop around the reference aim point and normalized crop offsets. Render the scene at final crop pixel density; do not crop roughly 608 pixels from 1920×1080 and upscale them to 1080×1920. A centered crop is a normal symmetric projection with a narrowed horizontal frustum. A 9:16 crop of a 16:9 reference at horizontal FOV 90° keeps about 35.12° horizontally while retaining its vertical FOV. Use an asymmetric/off-axis frustum for offset crops, not a camera turn. Both approaches can render directly at 1080×1920, avoiding the waste of an approximately 3414×1920 landscape intermediate.

Use a high-resolution landscape render followed by cropping as a test reference, with documented rounding/filter tolerances. Verify world geometry, viewmodel framing, sprites/beams and screen-space effects. Transform the viewmodel consistently with the reference crop, including any separate weapon FOV policy. Lay out HUD after cropping so vitals and kill feed remain visible. Crosshair position follows the transformed aim point; it may move off canvas for an offset crop. Freeze crop coordinates per job, and show excluded side action in preview.

Changing directly to a native portrait camera while keeping horizontal FOV 90° would produce about 121.28° vertical FOV; this is not the same as cropping a landscape view. Keep that as a separately named later experiment. Free cameras and automatic victim tracking need independent path/timing/occlusion design.

### 7.4 Color contract

Preserve the composed viewer's full-range display RGB values. Current `gpu::COLOR_FORMAT` is `Rgba8Unorm`, game gamma is already applied in shaders, and the live blit handles sRGB targets. A texture-format name alone does not identify its color encoding. Extract the final scene-plus-HUD display RGB without an additional game gamma or sRGB-inverse/BT.709-transfer pass. The export readback is opaque, top-to-bottom, tightly packed RGBA display bytes.

Use the conventional SDR capture compatibility path: RGB-to-YUV conversion with the BT.709 matrix, full-to-limited range, `yuv420p`, and BT.709 primaries/matrix/transfer metadata. The selected behavior preserves the viewer's display appearance; it is not a claim that sRGB and BT.709 transfer curves are identical. Metadata alone does not perform the matrix/range conversion. Keep one owner of that conversion (FFmpeg initially), and do not reintroduce a transfer correction in a shader or filter. OBS documents a similar compatibility treatment between these SDR spaces. [OBS SDR color handling](https://github.com/obsproject/obs-studio/wiki/High-precision-color-spaces-%28including-HDR%29), [FFmpeg scale filter](https://www.ffmpeg.org/ffmpeg-filters.html#scale).

Compare grayscale ramps, near-black detail, pure colors, transparent edges and HUD whites with the live viewer at identical settings. Check decoded output in a defined common RGB interpretation, browser playback, and an owner-authorized YouTube upload. Scene-only and scene-plus-HUD tests catch double encoding and alpha mistakes. Do not adjust game gamma by eye to hide a conversion error. Record the tested player/browser versions; do not promise identical color in every playback application.

## 8. Sound with exact synchronization

Create an independent offline `Playback` with the same immutable source snapshot, timeline, assets and initial state as the video pass. Do not record the system speaker, call `Device`, depend on CPAL callbacks, or use a virtual sound card. System mute and unplugged headphones have no effect on the file.

### 8.1 Active channels and bounded mixing history

The current live mixer advances playback to a block's end before mixing its channel snapshot. Stops/replacements/expiry may already have removed sounds heard during that block; a sound that both starts and stops within it can disappear completely. Volume is folded into spatial gains once per block, so a later volume change can incorrectly affect earlier samples. The device uses roughly 10 ms output blocks (40 ms of source time at 4×); smaller blocks alone do not fix the defect.

Keep the existing active-channel collection for stop/change lookup, channel replacement and eight-dynamic-channel allocation. **Do not leave retired voices in it.** Add a separate bounded interval history for mixing, with unique voice identity, session/group ordering, start time, exclusive end time, and timestamped volume/pitch changes. Every end path must retain the preceding audible span: explicit stop, named-channel replacement, channel stealing, natural sample expiry and session reset. Handle both network and client-generated/model-event sounds, delayed events and same-time ordering. Natural expiry uses sample length and integrated pitch; loops do not expire as one-shots.

Expose a block/interval API that returns every voice audible anywhere in that interval, including voices no longer active at its end. The mixer evaluates start/end and volume at each sample (or equivalent exact subspans), reusing pitch integration and loop sampling. Preserve all applicable same-time operations in file/event order. Convert event boundaries onto the output sample grid consistently: a change affects the first sample at or after its mapped time, with `[start, end)` inclusion. Use checked rational/integer mapping instead of independently rounded accumulated durations.

Release history after its consumer finishes the interval. The playback layer must also work without an audio consumer: history collection is opt-in/interval-scoped, with explicit capacity and cancellation rather than an ever-growing log. Each live/export consumer owns its cursor/history; seeks discard pending spans and reconstruct active state, and checkpoints do not retain consumed backlog. Compact long-lived volume/pitch history while preserving the integrated playback cursor. Snapshot enough spatial state that entity-slot reuse cannot relocate an ended voice to a different player retroactively.

Share this correction with the live viewer while retaining its small output blocks. Keep listener/source spatialization on an agreed source-time lattice of at most 10 ms at 1×, anchored independently of I/O block boundaries, plus relevant state changes. This motion approximation must be the same in live/offline/reference paths; camera sampling follows §6.3. Volume and stop/start times remain sample-accurate. Event-split production mixing is an acceptable fallback if substantially simpler after prototyping, but must satisfy the same interval API and tests; the chosen implementation must not change semantics.

### 8.2 Unclipped float spool and measured gain

Accumulate stereo samples without clamping and stream 48 kHz little-endian 32-bit float PCM into a private `.f32le` file. Twenty minutes needs about 460.8 MB (439.5 MiB), versus roughly 597 GB for uncompressed 1080p60 RGBA video. Use bounded buffers; never retain all PCM in RAM. Raw PCM avoids RIFF's 32-bit chunk limit. Diagnostic WAV may remain for short cases; RF64 is an optional self-describing alternative. Keep live-device protection separate from the unclipped export path.

Default user gain is unity (100%). Measure the unclipped assembled audio with FFmpeg `ebur128=peak=true` or a verified equivalent before integer conversion or lossy encoding. Prefer this existing meter over a new Rust true-peak implementation; an extra bounded-memory file scan is acceptable. True peak requires oversampling, not merely the largest stored sample. Reject nonfinite samples; handle silence explicitly. Record measured peak and gain. [FFmpeg EBU R128 filter](https://www.ffmpeg.org/ffmpeg-filters.html#ebur128).

For finite measured peak `P` in dBTP and selected user gain `U` in dB, apply a fixed protection gain `G = min(0, -1 - (P + U))` dB. The encoder audio filter applies `U + G` to the float spool. Thus protection only turns down, preserves relative dynamics, and targets a pre-encode ceiling of -1 dBTP. Measure across all joined clips and use one gain for the whole output; never normalize each cut independently. One extreme explosion can lower the whole file, so show the adjustment. The default is neither a limiter nor integrated-loudness normalization.

AAC/Opus can alter reconstructed peaks. Check decoded output in validation; a pre-encode ceiling is not an absolute post-codec guarantee. If a strict final ceiling is offered later, measure the encoded/decoded audio and retry with more headroom. A later optional limiter or loudness preset must specify latency, timing compensation and processing behavior; -14 LUFS is not a required YouTube upload level. Do not use `loudnorm` defaults when the intended behavior is a fixed gain, since its dynamic mode can change dynamics.

Generate exactly `S` sample frames, including the declared final silent padding. Include events at the selected start, exclude events at the end, and reconstruct ongoing sounds from initialization/checkpoints without restarting them. Audio at 30/60 fps is identical for the shared requested interval before final frame-quantization padding. AAC/Opus priming/padding must be checked with decoded click/flash fixtures and leading/trailing sample counts. Do not hide drift with `-shortest` or asynchronous resampling. Establish MP4 edit-list behavior empirically rather than disabling it blindly to follow an upload checklist.

### 8.3 Independent audio reference

Build a deliberately simple test-only event-split renderer: mix up to each state-changing sound event using the preceding state, apply it, then continue. Collect its transitions independently of the production history collector, observing the underlying playback transitions and scheduled events. If both paths consume the same incomplete history, agreement proves little. Natural expiry and looping still depend on sample/pitch semantics, not only explicit stop messages.

Use three layers of evidence: small synthetic cases with independently specified expected sample boundaries/amplitudes; differential comparison against the reference on selected real-demo intervals; and block-partition invariance with different I/O sizes (including 1, 127, 480 and 4096 sample frames on short fixtures). Cover all five end paths, starts/stops within a block, pitch/volume changes, equal-time operations, loops, clip starts, seeks and resets. Start with fixed listener/source fixtures, then compare moving cases on the same spatialization lattice so motion approximations do not create spurious failures.

Require exact sample counts and event boundaries; use a small justified absolute/relative float tolerance only for amplitude accumulation, with canonical voice order where possible. Compare unclipped PCM before gain/encoding, plus voice identities/boundaries to make missing paths visible. Matching two mixers establishes internal consistency, not parity with Half-Life; retain actual game A/V comparisons in release evidence.

## 9. FFmpeg execution, discovery and commands

### 9.1 Tool discovery and availability

Resolve an explicitly configured executable first, then an app-managed verified bundle if one exists, then PATH. Store an absolute resolved path and probe it outside the UI thread. Require ffprobe as well, preferably from the same distribution. Query version, build configuration, encoder/muxer/filter availability and run a tiny actual encode for the selected pipeline. An encoder listed by `-encoders` may still fail to initialize on the current hardware. Cache capability results by executable fingerprint and relevant device/driver identity, with Refresh.

V0 uses a known installed FFmpeg with Browse; do not build a downloader before the media path works. Before the general Windows release, provide a user-initiated **Download FFmpeg** action for an approved fixed build containing FFmpeg and ffprobe. Show provider/version/download size, use a versioned HTTPS URL and trusted pinned SHA-256, download to staging, verify before extraction/use, reject archive traversal, and atomically install into `%LOCALAPPDATA%\lambda-slicer\ffmpeg\<version>-<architecture>`. Keep versions separate and running jobs pinned to the resolved binary. Cancellation/failure leaves the prior installation usable. If the pinned artifact disappears, provide Browse/instructions rather than silently using an unverified latest build.

On Linux prefer a configured or system/package-manager FFmpeg, while keeping Browse and capability checks; do not assume every distribution enables the same encoders. Test the actual selected binaries. Bundling remains an alternative packaging decision with the same provenance/notices requirements. An upstream download does not by itself settle licensing, and an LGPL build containing only hardware H.264 encoders does not supply a portable software fallback.

FFmpeg's license depends on build options; enabling GPL components such as libx264 changes the FFmpeg build's licensing. Using a separate process keeps the integration boundary simple but does not remove redistribution obligations for that binary. Keep Lambda Slicer's own license choice separate, record the exact encoder build, include required notices/source arrangements before bundling, and reject a nonredistributable build. Do not claim that choosing H.264 or a subprocess settles every patent/licensing question. [FFmpeg licensing guidance](https://ffmpeg.org/legal.html).

### 9.2 Recommended first data flow

```text
frozen job + resolved record timeline
    ├─ audio playback → history mixer → temporary f32le PCM → peak measurement
    └─ video playback → shared scene/HUD compositor → bounded RGBA queue
                                                     ↓ stdin
                float PCM + measured fixed gain ──→ FFmpeg
                                                     ↓
                                      private partial output file
                                                     ↓
                                         ffprobe + decode checks
                                                     ↓
                                      final non-overwriting publish
```

Use `std::process::Command` with separate `OsString` arguments. No shell, interpolated command strings or user-entered filtergraphs. The following is an **argument template**, not a command to execute during planning. It assumes RGBA contains the unchanged display RGB specified in §7.4 and float PCM contains exactly `S` samples including tail padding. `<total-gain-dB>` is a validated finite numeric value computed in §8.2, serialized without locale-dependent formatting; it is not a user filter string:

```text
ffmpeg -hide_banner -nostdin -nostats -loglevel warning -n
  -f rawvideo -pixel_format rgba -video_size 1920x1080 -framerate 60 -i pipe:0
  -f f32le -ar 48000 -ac 2 -i <private-audio.f32le>
  -map 0:v:0 -map 1:a:0
  -vf scale=in_range=full:out_range=limited:out_color_matrix=bt709,format=yuv420p
  -c:v libx264 -preset medium -crf 20 -profile:v high
  -g 30 -bf 2 -flags +cgop -fps_mode passthrough
  -color_range tv -colorspace bt709 -color_primaries bt709 -color_trc bt709
  -af volume=<total-gain-dB>dB
  -c:a aac -profile:a aac_low -b:a 384k -ar 48000
  -t <effective-duration-T>
  -movflags +faststart -progress pipe:1 -f mp4 <private-output.partial.mp4>
```

Write exactly `N` frames of `W*H*4` tightly packed bytes, then close stdin. Raw input supplies the constant frame timestamps; preserve them rather than deriving timestamps from delivery speed. Change dimensions/fps/GOP coherently for each preset; GOP 30 above corresponds to 60 fps. Do not force an H.264 level incompatible with portrait, 1440p or 4K. No `-re`, screenshot delays, catch-up frame drops or automatic frame interpolation. If sound is disabled, omit the audio input/map/filter/codec settings and use `-an`. Do not use `-shortest` to conceal missing frames/samples. Validate option compatibility and exact decoded duration with the packaged FFmpeg version. [Raw media formats](https://ffmpeg.org/ffmpeg-formats.html#rawvideo), [FFmpeg process/progress options](https://www.ffmpeg.org/ffmpeg.html).

Drain stdout progress and stderr continuously on independent readers. Parse structured progress key/value blocks, bound retained log size, and keep unrelated text out of progress IPC. Test noisy stderr. Completion requires EOF, successful child exit, a complete mux, and verification; `progress=end` alone is insufficient. `+faststart` rearranges MP4 metadata after encoding, so retain a Finalizing phase and disk headroom. [MP4 muxing](https://www.ffmpeg.org/ffmpeg-formats.html#mov_002c-mp4_002c-ismv).

The output path in this template is an initially absent file inside the private job directory, not the final destination. Keep `-n`; do not pass a pre-created empty `NamedTempFile` path and expect FFmpeg to write it. After FFmpeg exits, adopt the completed staged file into the shared publication owner (§10).

For WebM use a separate typed command builder: `libvpx-vp9`, `-crf 30 -b:v 0`, `libopus` at 192 kb/s, `-f webm`, and supported encoder-specific threading options. Remove H.264-only profile/GOP flags, AAC options and MP4-only movflags. Capability-test this complete combination. No arbitrary container/codec combinations in the GUI.

### 9.3 Hardware encoders and alternative strategies

| Strategy | Benefits | Costs / decision |
|---|---|---|
| Software x264 through FFmpeg | Reference output, straightforward probing, works without a hardware encoder | CPU load; initial default. |
| FFmpeg NVENC / QSV / AMF; VAAPI where available on Linux | Can reduce CPU load and encoding time | Probe real initialization, supported resolution/pixel formats and driver errors. Quality controls differ; use encoder-specific presets and empirical comparisons, not identical CRF values. |
| GPU texture directly to hardware encoder | Avoids GPU→CPU→GPU copies | Requires platform/device-specific interop and ownership synchronization; defer until profiling proves readback is the bottleneck. |
| PNG sequence + PCM, then encode | Easy frame inspection and retrying encode | Huge disk usage/file counts; diagnostic short-range mode only, not the default for hundreds of demos. |
| Temporary lossless video + PCM | Re-encode/retry without rendering again | Large disk use and extra I/O; optional advanced workflow after the streaming path. |
| FFmpeg libraries via Rust bindings | Detailed timestamps and direct frame APIs | ABI/native build/ownership/distribution burden; reconsider only for a demonstrated limitation of the subprocess design. |
| Windows Media Foundation | Potential OS encoder integration without an FFmpeg download | Separate Windows-only backend, not initial choice. Hardware transforms must be enabled/probed and are not guaranteed; Windows N may need the Media Feature Pack. Does not remove all codec/license/distribution questions or supply Linux support. |
| All-intra H.264 | Independently decodable frames | Benchmark later against DNxHR/ProRes for quality, size and actual editor scrubbing; no promised middle-ground preset. |
| Capture Half-Life itself (`startmovie` or recording tools) | Reference for game fidelity | Game version/configuration/window/audio capture dependencies; separate optional backend requiring its own synchronization tests. Useful for validation, not the initial portable engine. |
| Real-time desktop recording | Quick manual workaround | Captures UI, stutter, interruptions and device sound; does not satisfy this exporter. |

A hardware failure must not silently switch encoder halfway through a file. Fail with Retry using software, or restart the whole job with a clearly reported fallback policy chosen before export. Audio preparation can be reused only when its job/source/assets fingerprint still matches.

Media Foundation background: [H.264 encoder](https://learn.microsoft.com/en-us/windows/win32/medfound/h-264-video-encoder), [hardware-transform enablement](https://learn.microsoft.com/en-us/windows/win32/medfound/mf-readwrite-enable-hardware-transforms). Evaluate it only if a concrete Windows requirement outweighs the extra backend; it is not a replacement for the agreed cross-platform FFmpeg path.

### 9.4 Size targets and ordered joining

Keep **Quality** (CRF, approximate size) and **Target file size** (explicit byte ceiling) as separate modes. For the latter reserve measured/conservative container overhead and safety margin, subtract audio bytes from the total budget, then derive a video bitrate from exact output duration. If the budget is impossible or likely poor, explain the tradeoff and let the user shorten the range or accept a proposed lower resolution/fps. A lower resolution under CRF alone cannot guarantee a limit. Never silently drop sound or change the selected range.

Use the selected software encoder's tested bitrate/two-pass mode for target-size jobs; isolate per-job pass logs. Streaming raw video means two-pass encoding needs deterministic re-rendering for the second pass. Begin with that disk-bounded strategy, report the extra rendering work, and reuse the audio spool. A bounded-budget temporary mezzanine can be an advanced alternative after profiling; never store hundreds of GB of raw video by default. Verify actual file bytes before publication. Permit a bounded retry with lower bitrate and reused preparation; if still too large, fail with an actionable retry choice rather than publishing an over-limit success. Encoder/muxer estimates are not a hard guarantee. Hardware size targeting needs its own verified strategy before exposure.

Joining selected clips is an explicit **One video with cuts** scope alongside separate files. Freeze their order in an edit list; start with clips from one demo and let the model carry source identity for future expansion. Reconstruct each span independently, but render/encode the entire output in one continuous video/audio stream. Do not encode separate AAC files and concatenate them with repeated priming gaps. State does not leak between cuts; carry sound within a source span, truncate it at its selected end, and reconstruct active sound at the next span's start. No automatic fades, crossfades or silence between clips.

Build cumulative span boundaries in output time and schedule the assembled video once, avoiding one rounded-up frame per clip. A cut appears at the first scheduled frame/sample at or after its boundary; report quantization and warn when a very short span contributes no video frame. Pad only the final assembled tail under §6.2. Use one gain for assembled audio. Reset camera interpolation at every cut. Validate noncontiguous/reordered spans, active loops and equal-time events at cuts. Any future chapters/subtitles use this mapping, not original demo times.

## 10. Resource limits, cancellation and file safety

- Start with one in-flight GPU frame for correctness. Then use a ring of three staging buffers and a small bounded encoder queue. Honor `COPY_BYTES_PER_ROW_ALIGNMENT`, strip padding, check RGBA/BGRA order and image orientation, unmap before reuse and associate each buffer with its frame index. Handle map/poll errors explicitly.
- At 1080p60 RGBA readback is approximately 498 MB/s (decimal); at 4K60 approximately 1.99 GB/s. A hardware encoder does not remove this cost. Measure decoding, posing, drawing, HUD, readback, conversion and encoding separately before optimizing.
- Bound checkpoints, decoded asset caches, GPU allocations, queues, sample buffers and logs. Use checked size calculations and device maximum texture dimensions. Streaming media buffers must not grow with duration; compact timeline/camera indices have explicit budgets and chunk/spill behavior (§6.3). A long demo may take longer to render than its running time; that is acceptable and must be reported honestly.
- Cancellation reaches preflight, group advancement, asset loading, audio mixing, GPU waits and FFmpeg. A writer blocked on a full pipe must be released by terminating FFmpeg before joining that writer. Never join an export thread on the GUI event path while it is still doing blocking work.
- Supervisor owns the process tree. On Windows use a Job Object or equivalent tested kill-on-close arrangement; on Linux use a process group plus explicit cleanup/parent-disconnect handling. Use hidden Windows subprocesses. Cooperative cancel comes first, followed by bounded termination of a stalled worker/process tree. Ensure launch failure and parent crash are covered, including the window between creating and registering children.
- Communicate cancellation over a control channel distinct from FFmpeg's raw-video stdin. Never write `q` into a pixel stream. Give finalization a separate timeout policy so a large fast-start pass is not mistaken for a hung renderer.
- Store temporary audio and output in a unique private job directory on the destination filesystem. Preflight writeability and free space; continue checking/reporting ENOSPC during the job. Budget PCM plus encoded output and finalization overhead. CRF estimates are not hard guarantees.
- From the first tester build, reuse/extract `slice/output.rs` publication semantics: private owned temporary output, completed writer, sync, validation/hash, then `persist_noclobber` or an equivalently tested no-replace operation. The existing `publish_new` callback supplies an open `File`, so add a path-producer/shared finalizer interface instead of forcing FFmpeg into it. FFmpeg writes an initially absent path inside an owned private directory with `-n`; wait for its successful exit and close writer handles, adopt the file into a cleanup-owning `TempPath`/equivalent, sync and verify it, then publish. The staging directory is on the destination filesystem. Verify Windows handle behavior and Linux publication. An early destination check is only a friendly rejection, never the race-safety guarantee. Test a destination created concurrently just before publication. Never overwrite a source demo, clip JSON, existing video or another job's partial file.
- Failed/cancelled files never appear as successful final outputs. Delete only files owned by that job; optionally retain diagnostics on failure. On startup, identify abandoned jobs by a manifest and mark Interrupted; do not claim resumability or recursively remove an arbitrary supplied directory.
- Preserve the existing source-snapshot protection as well as publication. Snapshot/hash each demo once, checking for mutation while copying, then make preflight, camera track, audio, video and retries read that exact snapshot. An original file changed after a verified snapshot does not invalidate that immutable input. Budget source snapshots in temporary disk use; cancellation must interrupt copying. Resolve assets into an export-owned bounded cache/manifest and detect inconsistent replacement before use; fail/retry rather than combine versions. Reports omit personal absolute paths unless detailed diagnostics are requested.

Default missing-content policy: missing/corrupt BSP or unrecoverable decode error fails the job; missing noncritical textures/sounds/models follows an explicit strict or best-effort choice shown in the export window. Best-effort must report what was substituted/omitted and finish as Completed with warnings. Preflight scans resources needed in the selected interval and reconstruction pre-roll; late-discovered failures follow the same policy. Do not substitute the preceding map, freeze gameplay for the rest of the video or output a spinner as a successful frame. Partial export up to a decode failure is a separate user-chosen range, not a silent fallback.

## 11. Step-by-step implementation

Each milestone produces runnable evidence and a reviewable change. The first experiment is real footage; synthetic tests then pin down behavior that visual inspection cannot prove. Preserve existing viewer and `.dem` behavior throughout. Do not mark the feature finished after the first MP4 plays.

### V0 — Ten-second real clip and end-to-end experiment

1. Read §2 and locate a local demo interval with motion, a gauss sound/shot and visible HUD. Record source identity and exact range. Use an installed known FFmpeg/ffprobe build.
2. Add the smallest runner boundary needed for private offline playback, fixed 30/60 fps scheduling, scene-plus-HUD rendering and audio. A background thread/private state is sufficient here; do not build process IPC, a downloader or a large settings UI first.
3. Export a 10-second H.264/AAC MP4 at landscape resolution. Preserve display RGB and spool float sound. Reuse/extract source snapshot and safe publication now, with FFmpeg's path-based writer adaptation. Retain diagnostics for known camera/audio defects instead of claiming parity before V1/V2.
4. Verify FFmpeg success, stream properties, full decode, frame count and basic A/V alignment. Add a tiny generated frame-number/flash/click fixture alongside real footage. Measure decoding, drawing/HUD, readback, conversion and encoding separately; note HUD size, colors and repeated camera poses.

Gate: a playable real 10-second video with audible sound, scene and HUD, plus a brief measured report. It renders independently of wall-clock playback/minimization/audio devices, does not overwrite existing files, and never publishes failed output. This is an engineering experiment, not the general release.

### V1 — Timeline, color and audio correctness

1. Finalize typed jobs/edit lists, source snapshots, exact frame/sample scheduling and group-bounded advancement. Add synthetic clock resets, equal timestamps, loading/map transitions and terminal fallback fixtures.
2. Establish the final display-RGB-to-BT.709-YUV contract with ramps, dark real scenes and HUD regression captures. Replace V0 shortcuts without introducing a transfer conversion.
3. Implement active-channel/history separation, per-sample start/end/volume semantics, unclipped streaming audio, true-peak measurement and fixed reduction-only gain. Use the same correction in the live viewer.
4. Add independently expected synthetic samples, event-split reference comparisons and block-partition invariance. Cover all voice termination paths, pitch/volume, scheduled events, loops, seeks and clip starts. Check decoded priming/padding and a long demo's beginning/middle/end sync.
5. Harden source mutation, validation failure and destination-collision tests for the shared publication path. Add generated-media FFmpeg integration tests that run without game/GPU/audio hardware.

Gate: exact `N`/`S`/group mapping, no lost same-block sounds, no cumulative drift, correct colors and no false success. Export at 30/60 fps shares the same unpadded audio. Keep existing audio/playback/slicing checks passing.

### V2 — Shared camera interpolation and playable preview

1. Build the anchored camera track in preflight and shared playback sampling. Add bounded separate-state lookahead for live playback; no full-scan startup requirement or future event leakage.
2. Implement break reasons and conservative holds, then verify reset-to-sample alignment on real footage. Cover respawn, recorder teleport, spectator target switch and watched-player teleport without a recorder `svc_setangle`.
3. Verify fast continuous AG movement does not falsely break. Test angle wrapping, equal times, sparse samples, EOF, FOV changes, session reset and seek/load invalidation.
4. Expose smoothing/recorded stepping and camera-update-rate information. Add a short playable export preview using private state and the shared compositor/audio, not only still frames.

Gate: smooth motion where samples permit it, no interpolation across verified discontinuities, no invented motion through walls, and no future kills/sounds/HUD applied by lookahead. Archive fixture ranges and reasons, not assumptions based on map names.

### V3 — Framing, portrait and export workflow

1. Implement separate framing/FOV, canvas and placement settings; add Fit and direct-resolution centered/offset crop. Compare direct crop with the high-resolution landscape reference, including viewmodels/sprites.
2. Add portrait HUD rectangles and width-based scale, background choices, privacy and overlay language. Verify mobile-sized preview readability and transformed crosshair position; guides never enter the file.
3. Add Export video entry points, range summary/suggestions, versioned preferences, frozen jobs, serial queue and completion actions. Commit clip edits before queueing; keep `.dem` output separate.
4. Exercise entire multi-session demos and selected clips, preview invalidation, changes to live viewer/demo/settings, minimization and close/cancel behavior.

Gate: intended ranges and framing, correct 16:9/9:16 output with readable HUD, and independent live/export state. Complete-demo mode preserves warmup/pauses/intermissions by default. Existing viewer captures and demo-slicing tests remain correct.

### V4 — Production worker and resource handling

1. Host the runner in a supervised headless process before general release. Dispatch before GUI/scans/audio devices; implement bounded progress/log IPC and cancellation without UI-thread joins.
2. Own the worker/FFmpeg process tree on Windows/Linux. Test broken/full pipes, noisy stderr, stalled workers, child/parent crashes, GPU loss/timeouts, disk-full and finalization delays.
3. Bound caches/history/indices/readback/logs, handle terminal asset failures, and enforce strict/best-effort missing-content policy. Test source snapshots and mixed-version asset rejection.
4. Profile a 20-minute 1080p60 run and short 4K sample. Add readback reuse/queues only where measured; preserve order and backpressure. Check active-viewer responsiveness and hardware limits.

Gate: controlled memory/disk use, reliable cancellation at each phase, correct cleanup/publication, and no orphaned processes. Record Linux GPU behavior as untested if only CPU/process integration was exercised; process isolation does not promise survival of a system-wide GPU reset.

### V5 — FFmpeg setup and first MP4 release

1. Add the approved pinned/checksummed Windows download flow with FFmpeg and ffprobe, separate installed versions, Browse fallback and cancellation. Keep Linux system discovery plus manual paths.
2. Probe actual encoding/filter/muxer capability, not only executable existence. Test a clean packaged Windows app without development tools and an unavailable download artifact.
3. Write the short user guide and `docs/video-export-validation.md`: setup, actual compatibility, portrait choices, privacy, disk/time estimates, editor caveats, versions and measured evidence.
4. Run `scripts/check.ps1 -Linux` where available plus exporter integration tests. Complete the MP4 acceptance rows; upload a representative sample only when the owner authorizes it.

Gate: ordinary Windows users can set up/export without a command line; a reliable MP4 release may ship. Keep TODO 57's multi-format portion open until V7. Downloader work must not delay the V0 real-clip experiment.

### V6 — Target sizes and joined clips

1. Add separate quality/target-size modes, editable byte budget, bitrate planning, explicit tradeoff proposals, actual-byte verification and bounded retry.
2. Implement software two-pass streaming via deterministic re-rendering; reuse frozen assets/audio and isolate pass logs. Measure extra cost and show it in progress.
3. Add frozen ordered plain-cut edit lists, continuous encoding, cumulative frame/sample scheduling and one audio gain for the assembled output.
4. Test reordered/disjoint clips, sub-frame spans, sound at cuts, privacy consistency and size overflow. Add square/4:5 canvases only after the same framing/HUD tests pass.

Gate: no over-limit output reported as successful, no accumulating per-clip padding or AAC gaps, and no state leaking across cuts. Separate-files remains the default scope.

### V7 — WebM and full multi-format acceptance

1. Add the independent VP9/Opus command builder and real capability probe; remove MP4/H.264-specific options and repeat color/sync/priming/cancellation tests.
2. Verify both delivery formats, landscape/portrait presets, clips, complete demos and the sharing workflows on the acceptance matrix. Update platform observations and limitations with dates.
3. Run release checks, document actual packaged encoder builds and keep Linux manual/GPU gaps explicit.

Gate: both advertised formats have evidence. Mark TODO 57 complete only after required clip/whole-demo/audio/preset/cancellation and multi-format behavior passes. Do not require later editing/creative features to close the core TODO.

### V8 — Editing output and measured optimizations

1. Add one editing preset: MOV/DNxHR HQ with uncompressed 48 kHz stereo PCM. Verify actual editor/version import, forward/reverse scrubbing, portrait resolution, colors and sample alignment; document the Resolve 20 Linux limitation without generalizing to every version.
2. Add hardware encoding one implementation at a time with actual initialization tests, explicit software retry and independently tuned quality. Enable 1440p/4K/custom sizes only after resource tests.
3. Benchmark all-intra H.264, ProRes or lossless intermediates only if a real workflow motivates them. Evaluate short/long GOP, file size and scrubbing on matched content/quality, not codec labels alone.

Gate: advertise only tested combinations. The later roadmap below is optional and separately scoped.

### 11.1 Later creative and posting features

Keep these outside the initial milestones: exported thumbnail from a selected moment; privacy-filtered title/description/metadata and chapters; chat as SRT rather than burned text; optional match cards, kill counters, watermarks and portrait zoom inset; batch filtered-kill compilations; explicit pause removal; slow motion, speed ramps, reverse output, motion blur and music mixing. Build them on the same edit-list mapping, compositor and privacy policy rather than special export clocks.

Chapters/subtitles must map through cuts, discarded spans and future speed changes. Apply platform chapter rules only to the platform-specific text helper (YouTube currently requires first 00:00, at least three chapters and minimum 10 seconds each); do not silently invent chapters that fit no event. [YouTube chapter guidance](https://support.google.com/youtube/answer/9884579?hl=en).

For motion blur, specify shutter interval/phase, sample times, behavior across cuts/teleports and HUD treatment before implementation. Supersample the scene with independently reconstructed states; composite a sharp HUD once. Define accumulation color space and prevent sampling outside selected source spans. It is not merely averaging arbitrary gamma-encoded screenshots. Music/time-stretch needs separate mixing, synchronization and user-facing rights choices; no automatic music or upload is included.

## 12. Acceptance and regression matrix

Apply each row when its feature milestone is reached. V5 requires the core MP4 rows; joined/size modes arrive in V6, WebM in V7, and the editing preset in V8. Deferred features do not block the first MP4 release.

| Area | Required cases and assertions |
|---|---|
| Timing | 1 frame; less than 1 frame; non-frame-aligned boundaries; exact boundary event; 30 and 60 fps; long duration; rational rates in pure tests; checked overflow. No frame loss/duplication and no end event leakage. |
| Sessions | Monotonic multi-map source; repeated/reset source times; server-time reset with continuing source time; loading records; intermission; empty/invalid range. Correct order and declared transition-duration policy. |
| State reconstruction | Clip starts during a looping sound, charging gauss, temp effect, decal, HUD message or weapon sequence. Starting directly matches the corresponding portion of sequential playback. |
| Audio | All five voice-end paths; same-block start/stop/replacement; timed pitch/volume; equal-time events; loops, seeks and resets; no device; moving listener/source; unclipped peaks, silence and decoded priming/padding. Independent expected samples, event-split reference and block-partition invariance. Exact sample counts/boundaries with justified float amplitude tolerance; click/flash offset at most one video frame and no increasing drift. |
| Camera | Respawn, recorder teleport, spectator target switch, watched-player teleport without recorder setangle, high-speed continuous AG movement, angle wrap, equal/sparse times, session/EOF, FOV changes and invalidated lookahead. Verify break-to-sample alignment, no future event leakage and bounded startup/memory. |
| Picture | Scene/HUD; transparency and missing assets; dark/ramp/color fixtures without a transfer pass; orientation and padded row stride; 16:9/9:16, both FOV policies, direct crop versus high-resolution crop reference including weapons/sprites; width-scaled portrait HUD, Unicode/RTL, privacy across every surface. Compare decoded RGB in a defined common interpretation and validate browser/upload appearance. |
| Joined/size modes | Reordered cuts, ongoing loops at cut starts, sub-frame spans, total-duration quantization, one audio gain/encode, no repeated priming gaps; exact byte limit, overhead, two-pass repeatability and bounded retries. |
| Editing preset | Actual named editor/version imports DNxHR/PCM MOV, seeks/scrubs both directions, decodes sound and portrait footage, and preserves expected color/timing. |
| Failures | FFmpeg absent/incompatible; encoder initialization failure; GPU device loss/map timeout; full disk; read-only destination; malformed demo; source/assets replaced; child crash; parent crash. No false success or overwritten user file. |
| Interaction | Edit/delete/reorder clips after enqueue; change demo, volume, settings or window size; minimize; close; cancel in every phase. Frozen jobs and live viewer remain independent. |
| Portability | Windows paths with spaces/non-ASCII/reserved names; Linux native paths; optional device/audio features; process-tree cleanup and concurrent filename collision. |
| Scale | 20-minute 1080p60 export and several queued clips with stable memory; stress unique assets/maps and measure cache growth. Cancel when encoder is slower than producer. |

Verification runs before no-clobber publication from V0 onward. It uses ffprobe JSON for codec/container, resolution, pixel format, sample rate/channel count, color tags, duration and timestamps, followed by decoding checks. Count decoded frames in integration/release tests; fully decode release fixtures to catch broken packets. Production verification inspects streams, decodes first/last frames and compares expected counts/durations with available encoder/container evidence; record when a count is metadata-derived rather than fully decoded. Offer full decode/count verification for stronger assurance and require it for suspicious output. Target-size jobs also check exact final bytes. Any validation failure prevents publication. Check decoded audio sample content separately from compressed packet durations. [ffprobe documentation](https://www.ffmpeg.org/ffprobe.html).

Determinism means equivalent media content/timing under fixed inputs, not universally byte-identical MP4 files across drivers/FFmpeg versions. Hash pre-encode frames and PCM on a fixed configuration; use visual/numerical tolerances across GPU backends. Capture with Half-Life for fidelity comparisons when available, especially gauss charge/release, a kill, HUD and map transitions. Keep source/asset identities in local validation records without redistributing the user's demos or game files.

## 13. First implementer task

Implement V0 first: a 10-second real-demo MP4 with synchronized sound, fixed scheduling, shared scene/HUD rendering, unchanged display RGB, immutable input and safe verified publication. Use installed FFmpeg and private background state. Bring the playable file, exact source/range/settings and stage timings back for review, with any observed camera/audio shortcomings listed for V1/V2. Do not spend the first iteration building the downloader or process framework.

Then proceed in milestone order, keeping a compact validation record as evidence accumulates. Locate actual camera fixtures before asserting coverage: the scratch-library `kz_cargo` demo is only a candidate for teleports until inspected. Record the source fingerprint, session/group/time, before/after camera poses, relevant messages/observer identity and expected break reason for each chosen respawn/teleport/spectator-switch interval. Include a watched-player teleport and a fast continuous movement counterexample.

Do not implement export by looping the screenshot command, recording the window, or passing a `.dem` directly to FFmpeg: a demo is recorded game state/messages, not an encoded audiovisual stream.

Source links above were consulted on 2026-10-23. Recheck platform requirements and the exact FFmpeg build when implementing/releasing. The command templates and architecture are design proposals; no encoder execution, real video export or platform upload was performed while writing this plan.
