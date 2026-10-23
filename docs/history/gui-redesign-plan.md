# Lambda Slicer GUI redesign implementation plan

## 1. Scope and source of truth

Implement the supplied Vercel design in the existing native Rust/egui application. The result must retain Lambda Slicer's analysis, real demo rendering and sound, seeking, clip persistence, and validated `.dem` export.

This is a presentation and interaction redesign, with one explicit analysis addition: evidence-based wallgauss identification for the new kill filters. Existing airborne/speed analysis must be exposed rather than reimplemented. Keep the existing Rust workspace, eframe/wgpu integration, playback worker, audio mixer, parser, and export engine. Do not introduce a webview, React runtime, service, or network requirement.

Baseline inspected on **20 October 2026**:

- Application repository: `C:\Users\naz\source\repos\lambda-slicer`, commit **`a5bf3fa`**, `Add match countdown HUD/sounds`.
- Live visual reference: <https://lambda-slicer-v0-2-0.vercel.app/>.
- Local reference: `C:\Users\naz\Documents\lambda-slicer-ui-redesign`.
- Current behavior: [gui.md](../gui.md), [gui-text.md](../gui-text.md), and [IMPLEMENTATION_PLAN.md](implementation-plan.md).
- Existing application assets: [assets/branding](../../assets/branding).

The local reference is a Next.js mockup with generated demo records, a static viewer image, a simulated playback timer, simulated scanning, and several buttons without handlers. Use its appearance and organization. Use the Rust application as the authority for actual data, calculations, supported file formats, frame boundaries, playback, and export restrictions.

The same-day design update adds a direct installation picker in the empty viewer, a Game executable setting, Midair/Speed/Wallgauss kill refinements, and kill badges. These requirements are integrated below; the corresponding TypeScript fields are sample data, not evidence that wallgauss detection already exists.

This plan adds stages **G0–G7**, including the **G4a kill-analysis gate**, separate from the existing M milestones. It does not declare any outstanding viewer-fidelity or game-comparison work complete. Preserve later application changes if the implementation starts after the baseline commit.

Follow this document in order: establish the component/state boundaries in §§2–6, implement the component behavior in §§7–12, then use the G-stage gates and verification matrix. The live site is a visual aid; the sizes, colors, behavior, and exceptions needed for implementation are specified here.

### Required result

1. A quiet dark library with collapsible grouped filters, compact status indicators, selectable columns, and clear scores and intensity.
2. Opening a demo replaces the large table with a narrow two-line list. A real viewer occupies the middle; Events, Clips, and Scoreboard occupy a separate inspector.
3. Transport and a compact timeline remain visible while inspecting any of those tabs. Detailed player lanes remain available.
4. Settings become a movable window with General, Library, Viewer, and Clips sections.
5. All existing analysis and export capabilities remain reachable, including when no game folder is configured.
6. Both Windows and Linux builds continue to work. Windows is the primary visual and interaction validation platform.
7. Users can find airborne, high-speed, and identified wallgauss kills using the new controls, with unknown observations distinguished from negative results.
8. A missing game installation has an immediate **Locate hl.exe…** action on Windows. The app still supports existing valid folder settings and native Linux installations.

## 2. Reference-to-Rust map

Paths in the second column are relative to `C:\Users\naz\Documents\lambda-slicer-ui-redesign`. GUI paths in the third column are relative to `crates/lambda-slicer-gui/src`.

| Area | Visual reference | Existing implementation and intended change |
| --- | --- | --- |
| Palette and typography | `app/globals.css`, `components/lambda/widgets.tsx` | Add `theme.rs` and `widgets.rs`; install the theme once from `main.rs`. Keep `text.rs` and `layout.rs` for localization and direction. |
| Application shell | `app-shell.tsx`, `app-chrome.tsx` under `components/lambda` | Reorganize `App::ui`, `toolbar`, `status_bar`, and `details` in `app.rs`. Retain drop handling and job ownership. |
| Library filters/table/list | `filter-panel.tsx`, `library-table.tsx`, `demo-list.tsx` | Split the presentation methods in `library.rs`; retain `Filters`, `FileQuery`, cached pages, refresh throttling, and native paths. |
| Demo layout | `demo-workspace.tsx`, `demo-header.tsx` | Split `DemoView::ui` in `demo.rs` into header, workspace, inspector, and timeline methods. |
| Transport and viewport | `transport-bar.tsx`, viewer section of `demo-workspace.tsx` | Split `Viewer::ui` and `Viewer::controls` in `viewer.rs`; continue using the existing playback controller and wgpu callback. |
| Timeline | `timeline.tsx` | Keep `timeline.rs` as the data/model layer. Add `timeline_ui.rs` for the overview, compact track, detailed lanes, and hit testing; extract existing painter code from `demo.rs`. |
| Events | `events-tab.tsx` | Reuse `Shown`, `refresh_rows`, `select`, `event_nav`, `describe`, and event-table data from `demo.rs`. |
| Clips | `clips-tab.tsx` | Reorganize `clips_ui`, `clip_row`, `time_field`, and `export_state`; retain `clips.rs` and `export.rs`. |
| Scoreboard | `players-tab.tsx` | Reorganize `DemoView::scores`; keep `standing_at` and coverage rules. |
| Settings | `settings-window.tsx`, `lib/settings.ts` | Extract UI into `settings_ui.rs`, using the existing serializable types in `settings.rs`. |
| Installation warning/picker | `viewer-setup-notice.tsx`, `demo-workspace.tsx`, `settings-window.tsx` | Add a shared picker/validation action in `app.rs` or a small `game_setup.rs`; extend `ViewerSettings` without changing the meaning of `game`. |
| Kill refinements and badges | `events-tab.tsx`, `lib/demo-data.ts` (`KillTricks`, `matchesTricks`) | Add `kill_filters.rs` in the GUI; carry `Kill.context` into `timeline.rs`. Existing motion is in `lambda-slicer-core/src/analysis/{motion,highlights}.rs`. |
| Wallgauss analysis | Random `wallgauss` values in `lib/demo-data.ts` are placeholders | Add a cancellable GUI enrichment job and a CPU-only classifier in `lambda-slicer-playback`; use recorded events, game-file callbacks, and source-state identity. See §10.1.5. |

The repository pins **egui/eframe 0.37.0**. It currently uses `eframe::App::ui` and `egui::Panel::{left,right,top,bottom}(...).show(ui, ...)`. Use that API, with nested panels and `CentralPanel`, rather than copying the mockup's older `SidePanel`/`TopBottomPanel` suggestions literally. `TableBuilder`, `ScrollArea`, `Window`, `ComboBox`, and `selectable_value` are already suitable.

Do not upgrade dependencies just to rearrange the GUI. Register new modules in `main.rs`. Introduce small typed presentation helpers rather than making all `App`, `DemoView`, and `Viewer` fields public.

## 3. Decisions where the mockup is incomplete

These decisions are part of the implementation specification.

| Mockup detail | Required native behavior |
| --- | --- |
| Static crossfire picture for every demo | Draw the actual wgpu viewer, retaining map errors, missing assets, playback stop reasons, countdown HUD, and audio. |
| Timer advances by 50 ms; frame stepping adds 10 ms | Controller time is authoritative. Step actual recorded groups with `Command::StepGroup`. |
| No visible reverse-play control | Keep reverse playback, including J/K/L shortcuts and its silent-audio behavior. Add a reverse-play icon to the transport. |
| Initial sample clip and fake event counts | Start from saved clips and actual model data. Never insert a sample clip or fabricate a count. |
| Clip padding forced to at least three seconds | Respect saved `preroll`/`postroll`, including zero; preserve validation of empty or unexportable ranges. |
| Three languages and 1–16 analysis workers | Keep all 14 languages, system language behavior, and the current 1–4 worker limit. |
| Different gamma defaults/ranges | Use `ViewerSettings::default()` and existing clamps, not `GAME_PICTURE_DEFAULTS` from TypeScript. |
| Recorded-FOV checkbox without a manual value | Retain the numeric FOV override when recorded FOV is disabled. |
| Accuracy filter stops at 60% | Retain 0–100%. |
| “Playing”/“Spectating” use different mock thresholds | Retain the real majority-of-recording definition: Playing ≤50%; Mostly spectating >50%. Explain it in the control tooltip. |
| Only blue/red teams, summed mock player scores | Use real teams/players, authoritative team-score updates, duel and FFA handling, and existing incomplete-statistics rules. |
| Shots/actions omit the actor in their row text | Always identify who used the weapon. This is central to finding Gauss footage. |
| Export buttons have no implementation | Route to the existing validated asynchronous exporter, including cancellation, refusals, and playable-path instructions. |
| A tooltip claims every cut is only milliseconds longer | Show the actual requested/effective bounds from `GroupIndex::snap`; make no fixed rounding guarantee. |
| No detailed player lanes | Offer “Player lanes” below the compact timeline, preserving connection identities and weapon spans. |
| Long messages hidden in help icons | Put explanatory coverage notes in help. Keep failures and conditions requiring action visibly discoverable. |
| Browser fonts, generic lambda, and website analytics | Keep existing approved branding and native text support; ship no website analytics or runtime font downloads. |
| “The viewer runs demos through hl.exe” | The viewer is native Rust/wgpu. Selecting `hl.exe` locates its asset installation; do not launch/embed the executable or require it on Linux. |
| Missing `hlExePath` always blanks the picture | Existing valid `viewer.game` and `--game` installations remain usable. Show setup when usable installation assets are unavailable, not just because a new preference is missing. |
| Airborne values converted with `!!`, missing speeds replaced by zero | Preserve optional values and motion evidence. Unknown is neither grounded nor stationary. |
| Random wallgauss boolean | Only supported evidence/reconstruction may produce a WALL badge; incomplete data stays Unknown. |

Change the fresh library sort to **Intensity descending**, matching the mockup and the footage-finding workflow. Sorting and filters must survive opening/closing demos and switching table/list presentation within the session. Do not reset a sort the user has already chosen.

“Back to library” and “Close demo” both save pending valid clip edits, close the current demo using the existing cancellation policy, and return to the full table. Hiding the compact list only changes layout; it does not close or reload the demo.

## 4. Visual system

### 4.1 Colors

Create named constants and a single theme installation function in `theme.rs`. Use these values from `app/globals.css`:

| Token | Color | Usage |
| --- | --- | --- |
| Background | `#17191c` | Main application and list backgrounds |
| Surface | `#1e2125` | Toolbars, settings, clip cards, timeline background |
| Raised | `#272b30` | Inputs and ordinary buttons |
| Hover | `#31363c` | Hovered interactive surfaces |
| Border | `#2b2f35` | Subtle separators |
| Foreground | `#d8d5cf` | Main readable text |
| Muted | `#8b9097` | Secondary labels and metadata |
| Faint | `#5e636a` | Decorative grid lines and low-priority metadata only |
| Accent | `#e8913a` | Primary actions, active selection, playhead |
| Accent soft | `#e8913a`, alpha `0x24` | Selected rows and toggles |
| Accent foreground | `#1a1206` | Text/icons on solid accent |
| Team blue | `#74a7de` | A known blue side |
| Team red | `#e0726a` | A known red side |
| Success | `#86b46e` | Successful analysis/export |
| Warning | `#d9b64f` | Partial coverage and actionable warnings |
| Error | `#d9675f` | Failure indicators |

Use egui visual fields for panel/window/input fills and widget states. Draw component-specific soft selection and fine borders where the global selection style is insufficient. Convert the translucent accent with an unmultiplied RGBA constructor.

The accent is amber; team colors and error colors retain semantic meaning. Do not recolor existing weapon-color mappings into amber. Do not make meaningful event timestamps, editable values, or error messages faint just because the browser mockup does.

Selected rows have both a soft fill and a 2-point leading accent edge. Focus has a visible outline. Status and team identity must have text/tooltips or symbols as well as color.

### 4.2 Sizes and typography

All dimensions below are **egui logical points**, not framebuffer pixels:

- Main text 13; secondary and numeric text 12; small headings 11; large score numbers 24.
- Retain the current native font families and bidi/shaping support. Match sizes, spacing, and weights before considering additional font assets.
- Standard control height 28; icon hit area at least 28×28 even when the glyph is 14–16.
- Main Play button 32×32, circular, filled amber.
- Main top toolbar 44; normal status strip 24; pane headers 44.
- Table rows 32; compact demo rows 48; event rows 28 where a single line fits.
- Spacing scale 4, 8, 12, 16. Default panel content padding 12.
- Corner radius 4 for small widgets, 6 for controls, 8 for cards and the settings window.
- Separators 1 point. Use few borders and no alternating zebra stripes by default.

These are preferred dimensions, not clipping constraints: localized labels and actual font metrics may increase a row's height. Use monospace for times, paths, percentages, and scores. Align numeric columns consistently.

Add painter-based line icons in `widgets.rs` for the required small icon set, using native egui responses and accessible labels. Do not rely on emoji or platform-dependent Unicode glyphs. If a bundled icon asset library is chosen instead, keep its license and use it consistently; do not add a web icon runtime.

Every icon button needs a localized tooltip and widget/accessibility label, a disabled reason when applicable, and a visible selected state for toggles. Help icons must also open the explanation on click or keyboard activation; hover must not be the only way to read it.

## 5. Application layout and sizing

### 5.1 Full library

```text
┌ Brand   Open demo   Add folder   Rescan/Cancel                     Settings ┐
├ Filters · count ┬ Results count                         Columns ──────────┤
│ Find           │ Demo       Date  Map  Length  Score  …                  │
│ Point of view  │                                                        │
│ Quality        │ Virtualized, sortable table                            │
│ Recorded       │                                                        │
├────────────────┴────────────────────────────────────────────────────────┤
│ Actual scan/cache status                         Selected/source path   │
└─────────────────────────────────────────────────────────────────────────┘
```

The filter pane is 248 points by default, resizable between 220 and 340. The Filters control remains reachable after collapse and displays the active-filter count. Its contents scroll independently when the window is short.

The table header stays above the table's own scroll area. If selected columns cannot fit, provide horizontal scrolling with the header and rows moving together. Do not silently remove visible columns to fit the window.

The status strip remains present even when idle. Obtain counts from the cache/job state, not from loaded page length. Show filtered count separately from total count. During scanning, retain progress, cancellation, rejection notices, and access to the existing problems window. A multi-line actionable notice may expand above the normal strip.

### 5.2 Demo workspace

```text
┌ Brand   Open demo   Add folder   Rescan/Cancel                     Settings ┐
├ Demo list ──────┬ Demo header                         ┬ Events Clips Score ┤
│ Search / sort  │                                     │ Local tab controls │
│ Compact rows   │ Real viewer                         │                    │
│                │                                     │ Scrolling content  │
│                ├ Transport ──────────────────────────┤                    │
│                ├ Compact timeline + overview ────────┤ Sticky tab actions │
│                │ Optional detailed player lanes      │                    │
├────────────────┴─────────────────────────────────────┴────────────────────┤
│ Actual background-job status / persistent notices                        │
└─────────────────────────────────────────────────────────────────────────┘
```

Default compact library width: **280**, range 240–360. Default inspector width: **360**, range 320–520; use 400 initially at content widths ≥1600. Preserve the user's chosen sizes within available-space limits.

The viewer receives all remaining middle space. The transport and timeline are reserved before allocating the viewport. Resizing a pane changes the callback rectangle and rendering aspect, not playback state.

The compact timeline uses about **124 points** including heading, a 76-point track, and a 12-point overview. Detailed lanes add a separately scrollable area, initially 180 points and capped so the transport and compact overview never disappear. The overall workspace must not be one giant vertical scroll area.

Use the existing renderer's viewport/aspect behavior. Do not force the browser screenshot's black bars or crop the weapon to imitate that screenshot. Preserve explicit capture/benchmark output sizes independently of the visible GUI size.

### 5.3 Responsive rules

Calculate layout from the current content rectangle in logical points. A width threshold is a starting policy, not permission to overlap controls when fonts or manual pane sizes need more room.

| Available width | Layout |
| --- | --- |
| ≥1180, with ≥500 points left for the middle | Three panes: compact library, viewer, inspector. |
| 960–1179, or three panes would leave <500 in the middle | Automatically collapse the compact library. Keep viewer + inspector. A header button opens the library as a temporary overlaid drawer. |
| <960 | Library becomes a drawer. Put the inspector below the middle viewer/timeline, with a resizable height, initially 220. Keep tab header and action footer fixed inside it. |

At narrow widths the transport may wrap to two deliberate rows. Keep start, group-step, play/pause, and current time visible; move volume, reverse, see-through, and picture settings into a clearly labeled overflow menu if necessary. All remain keyboard-accessible. Do not squeeze numeric time text until it becomes unreadable.

In a short window, reduce the viewport first; collapse expanded player lanes next. Keep at least a 120-point viewport when space permits, the compact timeline, and the active inspector controls. At 900×600 the normal workflow must remain usable. If even that space is unavailable, scroll the inspector/settings content instead of clipping actions.

Automatic collapse is temporary and must not overwrite the user's saved pane preference. When the window grows, restore their preference. Use a small hysteresis, e.g. 32 points, to prevent collapse/expand oscillation during resizing.

Mirror application side panes and leading edges for Arabic using the existing layout helpers. Keep time axes, transport time readouts, and numeric range editors left-to-right. Do not reverse the meaning of forward/backward icons.

## 6. State ownership and frame processing

### 6.1 Preserve the existing owners

- `App` owns settings, library, active demo, scan job, persistent clip store, and launch overrides.
- `LibraryView` owns the query, sort, page cache, and selection.
- `DemoView` owns the analysis model, event selection, timeline viewport, clips, export state, viewer, and kill-enrichment job/results.
- `Viewer` owns the controller, decoding worker, GPU resources, sound device, and draw state.

Tab switching, panel collapse, window resizing, and settings navigation must not construct a new `DemoView` or `Viewer`. Only a real source change or an existing required game-folder reload does that.

Introduce presentation state with explicit names, for example:

```rust
enum InspectorTab { Events, Clips, Scoreboard }
enum SettingsSection { General, Library, Viewer, Clips }

struct WorkspaceUi {
    inspector_tab: InspectorTab,
    compact_filters_open: bool,
    // Pane preferences and other transient presentation state.
}
```

Keep `Shown` as the sole event-filter state, extending it with `KillFilters` (§10.1.1). Keep `DemoView::view` as the sole timeline viewport, `selected` as a model event index, and `chosen_clip` as the selected clip. Avoid duplicate selections in tabs and timeline widgets.

Use stable egui IDs derived from component role and native demo identity. Event row IDs use the model event index plus demo generation, not a visible row number. Clip rows require stable **runtime** IDs: assign one when loading/creating a clip and keep it alongside the stored clip when removing/reordering. Do not use name or time bounds as IDs because both are editable. Runtime IDs need not change the `clips.json` schema.

### 6.2 Split work from visible widgets

Currently `DemoView::ui` polls analysis and exports, and `Viewer::ui` polls playback/assets, feeds sound, handles shortcuts, paints the viewport, and draws controls. Extract these responsibilities before introducing tab-dependent drawing.

Use an explicit frame sequence:

1. Drain scan results and refresh stale library state.
2. Resolve a requested demo change, flushing valid pending clip edits and saved clips from the previous demo before dropping it.
3. Apply current settings and launch overrides; poll analysis, playback, assets, and export results **once** for the active demo.
4. Route keyboard actions once, honoring focused controls, settings, popups, and active drags.
5. Draw pane contents and collect actions with stable targets.
6. Commit text edits before resolving actions that depend on their values. Apply actions, then synchronize loop bounds and feed audio from the updated controller state.
7. Persist dirty clips/settings and request repaint when needed.

The exact helper names may be `begin_frame`, `viewport_ui`, `transport_ui`, and `finish_frame`. They must remain methods on the owning types or small adapters, not new controllers.

Maintain existing capture, benchmark, timed audio-play, decal update, displayed-frame tracking, and countdown behavior when extracting the viewer. Audio must continue when the Clips or Scoreboard tab is active. A hidden library must not prevent scan completion or final cache refresh.

A play/seek action can repaint the viewer on the next egui frame; issue `request_repaint` rather than painting the viewport twice or advancing a second clock. Initialize the controller/decoder independently of game-asset availability so transport, time, group stepping, selection, and clipping work in setup mode (§8.4). Picture, sound, and asset-dependent kill enrichment remain unavailable until the installation resolves.

### 6.3 Presentation settings and migration

Add a `#[serde(default)]` nested UI-settings structure in `settings.rs` for filter-pane visibility, compact-list visibility, preferred pane sizes, and player-lane expansion. Clamp invalid/nonfinite sizes on load. Give new enums safe fallback deserialization so an unknown presentation preference does not discard valid game paths or clip padding.

Retain existing `hidden_columns` keys. The Demo/name column is essential; normalize an old hidden-name preference to visible. The old `status` key may control the embedded status indicator rather than a separate wide column. Keep `gauss` hidden by default but available from Columns.

The compact timeline is always visible. Migrate legacy `show_timeline` to **initial player-lane expansion** when the new UI field is absent: `true` expands lanes; `false` leaves the compact timeline. Do not interpret old `false` as hiding the new timeline. Preserve `--timeline` as “expand player lanes” and document it. Use explicit migration logic or an `Option<bool>` for detecting a missing new field.

Keep clip serialization, export-folder interpretation (`None` means beside source), locale, existing paths, analysis-worker count, padding, and existing viewer-setting meanings unchanged. Add only the optional executable locator described in §8.4. Panel/filter state need not acquire a new cache schema. Filters and sort remain session state; column visibility remains persistent.

Only save after a value changes. Sliders update the viewer live but save on pointer release, focus loss, or window close, retaining the existing batching behavior. Screenshot/capture launch overrides must not become saved user preferences.

## 7. Library implementation

### 7.1 Filters

Replace `filters_ui`'s wrapping toolbar with a reusable vertical filter form used by both library mode and the compact-list filter drawer:

| Group | Controls and bindings |
| --- | --- |
| Find | Search paths → `Filters.text`; Player → `player`; Map → `map`; Weapon → `weapon`. |
| Point of view | Any / Playing / Mostly spectating → `Spectating`, preserving current thresholds. |
| Quality | Minimum minutes, minimum accuracy 0–100%, Has headshot kills. Preserve existing query semantics and explanatory tooltips. |
| Recorded | Inclusive From/To bounds using existing `Ymd`, locale date ordering, and valid month lengths. |

Use a search-style editable Map field with optional suggestions, retaining substring search. A fixed list from the mockup would remove arbitrary/custom-map search. If suggestions are implemented, query distinct cached map names once per cache refresh and support typing; never hardcode `MAPS`.

Keep the current supported weapon list and unknown-value handling. Reset clears all filters together. Count active controls using their effective values: whitespace-only text is inactive; From and To each count once.

Use the existing `FileQuery` path. Do not move filtering or sorting into a `Vec` of currently loaded rows. Changing filters must not close the open demo even if it disappears from the results. Preserve the compact list's sort/search controls when returning from its filter form.

### 7.2 Table, columns, and cached data

Split `LibraryView::ui` into shared query preparation plus full-table, compact-list, and filter methods. Continue using 100-row cached pages and one-second scan refresh throttling.

Draw:

- A status symbol beside the filename, replacing the wide Status text column.
- Filename without the final `.dem` for display; full native path available through details.
- Date, Map, Length, Score, optional Gauss, Kills, HS, Accuracy, Intensity, Spectating.
- A Columns menu with the same canonical keys as existing settings. Preserve the existing header context menu as an optional shortcut.
- Hover, keyboard focus, and selected-row styling; Enter opens the focused row.

Suggested initial widths: Demo 280 (flexible, minimum 180), Date 112, Map 128, Length 80, Score 128, Gauss/Kills 64, HS 56, Accuracy 80, Intensity 144, Spectating 96. Let users resize columns. Use a common horizontal scroll container if their total exceeds the available width.

Map all real analysis statuses, including complete-with-no-shots, partial, unsupported, and failed. A partial result must not look like a complete successful analysis. Use a distinct outlined/different-shape symbol where colors alone would be ambiguous; retain the full localized status and stored error in the tooltip/details. Do not invent a per-file “pending” state when only overall scan progress is known.

Zero and unknown are different: show `0`, `0%`, or `0.0` for known zero values and a dash for unavailable values. Date tooltips retain the recording-date source. Accuracy remains recorder accuracy; Gauss remains the actual Gauss count.

#### Compact score data

The current `FileRow.score` is a shortened string and cannot reliably provide the mockup's `+N` count. Do not split that string to infer teams or side counts.

Expose the already-stored `score_json` through a typed optional score summary in `FileRow`, reading it in the existing paged query. Add deserialization for `analysis::context::{Scoreboard, ScoreSide}` or a compatible private read DTO. Preserve the CLI JSON contract by marking a GUI-only new field skipped if needed.

Render the first two ordered sides as `170 : 155`; append `+2` only when there are exactly two additional sides. The `+N` tooltip says “N more sides” or its team/player-specific equivalent; it is not a score difference. Full side names and scores belong in the tooltip/details. Handle one side, negative scores, no scores, and malformed/missing cached JSON. On missing structured data, show the legacy summary without inventing `+N`.

The mockup also makes Score sortable. Add `Sort::Score` for the first ordered side's signed frag count, using guarded extraction from `score_json` in SQLite; `json_valid` must protect malformed values. Match existing null ordering and deterministic tie breakers. This is a small query addition, not a reanalysis or schema rewrite. Add a cache test for negative values, equal values, missing values, and paging.

#### Intensity bar

The text uses `FileRow.best_window`, the existing action-window score. Do not recompute the score or use mock random values.

Use a fixed 64-point bar next to the number. Normalize the bar against the maximum finite positive `best_window` in the current-version cache, obtained with cached aggregate statistics on refresh, not a query per row. Keep this denominator independent of filtering so bars do not change meaning when a filter is applied. Clamp fills to 0–1; retain negative/zero numeric values with an empty bar and null as unknown. The tooltip identifies the real score and explains that the bar is relative to the library maximum.

### 7.3 Compact list

Use `ScrollArea::show_rows` or equivalent virtualized fixed-height rows. First line: status and filename. Second line: map, length, and compact score. Hover/clickable details provide date, player, intensity, accuracy, and path without widening every row.

The header has Back to library, result count, Filters with active count, and Hide list. The next row has Search, Sort by, and ascending/descending. Include **all** real sortable columns, including Gauss, HS, Spectating, and the new Score sort, rather than the mockup's reduced sort menu.

Selection and row lookup use `FileRow::source()`/native bytes. Display strings must never become file identity. Two same-named files in different folders remain separate. A demo opened directly remains open even if the cache is missing or unavailable.

## 8. Demo header, viewer, and transport

### 8.1 Header

Show the filename prominently, with map, date, duration, and recorder/POV as secondary metadata when space permits. A details/help popover retains everything currently shown by `App::details`: raw path, recording kind, shots, kills, intensity, coverage, and stored error. Missing metadata remains unknown, never taken from the previously open demo.

Actions: restore hidden list, Copy path, Show in folder, Close demo. Copy the complete path, not its truncated display.

Implement Show in folder through a small platform helper, using OS arguments without a shell-interpolated command. On Windows reveal/select the file in Explorer; on Linux open the parent folder through the desktop opener, with a visible failure message and Copy path fallback. Do not block the GUI while waiting for a file manager to close.

### 8.2 Transport

Order at normal width:

1. Start, reverse play/pause, previous recorded group, primary play/pause, next recorded group.
2. Current time / duration, using the existing source-time mapping.
3. Rate: 0.25×, 0.5×, 1×, 2×, 4× through the existing `Rate` enum.
4. See-through toggle, mute/volume, and Picture & viewer settings.

The current-time field may remain a readout; seeking is through the shared timeline. Remove the old duplicate seek slider once the new timeline is wired. Keep precision in controller values; display formatting does not alter them.

Mute remembers the most recent nonzero volume **for the session** and restores it on unmute; if none exists, use the current native default, not the mockup's hardcoded 0.6. The volume slider and Settings edit the same `ViewerSettings.volume`. “See through walls” edits the same setting and respects the items/projectiles sub-option. The picture button opens Settings directly on Viewer.

Preserve viewport click-to-play/pause, reverse direction resume behavior, source-time stop bounds, actual group stepping, seek cancellation, and audio generation invalidation. Do not send seek commands when merely panning or zooming the timeline.

### 8.3 Viewer states

Allocate the same viewport region in all states:

- No usable game installation: the centered setup warning and direct file/folder picker in §8.4, plus Settings → General.
- Loading/indexing/seeking: progress indication from available real state; do not promise a percentage the worker cannot supply.
- Missing assets: visible warning affordance opening the existing missing-file list.
- Map failure: map name and actual reason, with an appropriate settings action.
- Playback stopped early: a visible stop message, with details available.
- Ready: normal render, including countdown overlay and sound.

The rest of the demo workspace stays usable when viewing is unavailable. A loading or failed viewer must not hide analysed events or disable valid exports merely because rendering failed.

### 8.4 Direct installation setup

**Architecture:** `App::open_viewer` currently passes a Half-Life installation directory to `Viewer::open`. `Viewer` constructs an asset `SearchPath`/`Library` and renders internally. `hl.exe` is a convenient installation locator on Windows, not the rendering engine or a file to execute.

In the viewport, show a centered warning icon, heading “Set up the demo viewer”, short text explaining that game files are needed for picture/sound, a primary **Locate hl.exe…** button on Windows, and a secondary **Settings** button opening General. Below, explain that timeline, events, and clips remain available. At narrow widths wrap the buttons. Use localized copy; remove the inaccurate “runs through hl.exe” wording.

Both Locate and General's Game executable Browse invoke **the same real picker action** using `rfd::FileDialog::pick_file()`. Default the directory to the currently configured installation when one exists. On Windows filter for executable files and validate that the chosen regular file is `hl.exe` (case-insensitive) rather than a launcher shortcut or unrelated `.exe`. Check the parent installation layout, including the `valve` directory and the normal asset search. A missing optional custom map/model is a later asset warning, not a reason to reject the installation.

After a successful choice, set `viewer.executable: Option<PathBuf>` and derive `viewer.game` from its parent. Preserve both full native paths. Keep `viewer.game` as the canonical asset root already used by the renderer. Do not store a filename string in the existing directory field.

- Cancel leaves previous settings and viewer state unchanged.
- Invalid/unreadable selections show a reason next to Locate/Browse and do not replace the last working configuration.
- Validation/path lookup must not synchronously traverse the entire installation or probe disk every frame. Represent Checking, Ready, Missing, and Invalid states explicitly.
- Saved folders migrate without user intervention. If `<saved root>/hl.exe` exists on Windows, it may populate the new locator; if it does not but assets are usable, continue rendering and describe the executable as optional for that existing asset setup.
- Keep the Half-Life folder picker as an alternative, especially on Linux. A valid asset-only installation must not be rejected solely for lacking `hl.exe`.
- Linux's primary label is **Locate Half-Life…**, using a folder picker; an optional executable picker can accept the native `hl_linux` file and derive its parent. Never require a Windows executable, invent one, or require Wine.
- `--game` retains precedence for this run. Show that override explicitly and do not silently change the active root by browsing a different saved setting. Explain when the saved choice will take effect.
- If the saved executable later disappears but the asset root remains usable, warn in Settings without blanking a working viewer. If the root is missing, show setup with Relocate.
- Choosing a different asset root through the folder picker clears an executable locator from another root; all setters share this consistency rule.

General shows **Game executable** with a warning-colored “Not set” and highlighted Browse only when setup is genuinely needed. A valid legacy/Linux asset configuration must not claim “the viewer is disabled”. The existing Half-Life folder row remains visible as the resolved asset root and alternative setup route.

**Transport without assets:** refactor `Viewer::open` to accept an optional install root and instantiate the controller/worker even when it is absent. `Worker::spawn_with(..., None, ...)` already supports playback without game files. Skip GPU scene presentation, asset loader, and sound-device setup until assets are ready; avoid the current no-map spinner in this state. Native play/pause/rate/reverse/group-step commands continue to advance source time and update the timeline, silently and with the warning still displayed. Disable asset-only controls such as see-through with a reason. Never introduce a mock timer.

When setup succeeds, preserve source position, rate, direction, play/pause state, active preview end, and loop range. Rebuild asset-dependent playback resources/checkpoints if necessary, seek the new worker to the captured position, and resume only after the new generation is ready. Clips, event filters, and selection stay intact. Switching demos during setup must discard the old setup/resume target. Do not reuse checkpoints generated without assets in an asset-backed simulation if their derived effects differ.

## 9. Shared timeline

### 9.1 Model and drawing

Keep `Model.start/end`, `Model.events`, `LaneKey`, marks/spans, and `GroupIndex` as the underlying data. The full extent must include quiet tails and demos with no weapon events; use the existing model extent, not the last event time.

Draw the compact surface in this order:

1. Background and ruler/grid.
2. Activity-density strip.
3. Highlight interval and clip bands.
4. Event markers.
5. Selected event outline and playhead.
6. Clip handles/labels in their own strip so they remain usable.

Use one shared time-to-x transform for the compact track, detailed lanes, playhead, and clip edges. The overview uses the full recording extent and shows the current visible-window rectangle.

The density strip is **event density**, not the library's action-score formula. Aggregate unique visible `Model.events` into time buckets; a kill's killer/death lane marks must count as one event. Rebuild derived density only when the model, `Shown` (including kill refinements), or enrichment revision changes. Use a bounded base histogram and coarser summed levels, selecting a level appropriate to pixel width; do not iterate every event for every painted pixel or every frame. Normalize consistently across the whole demo for the current filter and explain the quantity in a tooltip.

Compact markers use known team colors when available, with neutral color for unknown actors and other teams assigned a stable palette. Distinguish kill/action/shot/switch shapes or rows. Headshots may be taller or accented, but unknown is not false. At wide zoom, aggregate crowded marks rather than silently dropping all shots at an arbitrary four-minute threshold.

Prepare any missing actor/victim identity and team-at-event metadata in `Model::build` while the `AnalysisReport` is still available. The current kill event stores a victim name but not its full identity; add a victim `LaneKey` or equivalent optional identity for display mapping. Resolve team membership from the report's identity/score-update data at that event's segment/time, not the final scoreboard or a name lookup. This is a presentation-model extension, not a change to kill attribution.

“Player lanes” expands the existing per-connection lanes below the compact surface. Retain recorder identification, unknown-actor lane, shot/action/kill/death/switch shapes, weapon colors, Gauss charge and Egon spans, and known/unknown span endpoints. Reused slots and changed sessions remain distinct `LaneKey`s.

The timeline heading has a small contextual legend, visible range, Player lanes toggle, Zoom out/in, and Fit. Detailed legends/help can live in a popover. Do not show “Blue kill / Red kill” when the demo does not have those known sides.

### 9.2 Gesture contract

Extract hit testing into helpers that return a semantic target before applying an action. Use this precedence:

1. Clip edge handles within a six-point hit slop; selected clip wins ties.
2. Playhead/ruler scrub zone.
3. Nearest event marker in the marker region.
4. Clip body/label in its dedicated strip.
5. Empty background.

| Gesture | Result |
| --- | --- |
| Click event marker | Select the model event, seek once, bring its row into view, activate Events. |
| Click empty track | Seek to that time without fabricating an event selection. |
| Drag ruler/playhead | Use existing begin/preview/end scrub behavior; preserve Escape-to-cancel. |
| Drag a clip edge | Edit that clip at whole-millisecond UI precision, update active loop bounds, and retain existing preview behavior. No background pan. |
| Click clip body/label | Choose the clip and activate Clips. |
| Drag empty track beyond a small threshold | Pan the visible interval; do not also seek on release. |
| Wheel over track | Zoom around cursor time. Consume the wheel only over this surface. |
| Zoom buttons | Zoom around visible playhead, or viewport center when playhead is outside. |
| Double-click empty track or Fit | Restore full extent. Do not intercept double-clicks on event/clip targets. |
| Drag overview window | Pan while preserving the initial grab offset. |
| Click overview outside window | Center the current viewport there, clamped to bounds. |

Keep a distinct `DragMode` for scrub, clip edge, pan, and overview pan. Release/cancel once. Moving over a different region mid-drag must not change modes. Esc first cancels that drag, not the demo.

Use the native `MIN_SPAN` of 0.5 seconds where the recording is long enough, not the mockup's eight-second floor. If duration is shorter, use that positive duration; handle zero/unknown duration explicitly with disabled seeking and no division by zero. Clamp all panning/zooming to actual bounds and reject nonfinite values. Fit must not force negative start times for short recordings.

Routine playback moves the playhead without replacing event selection or repeatedly scrolling the event list. When it leaves a zoomed viewport during forward/reverse playback, page the viewport just enough to keep it visible, except while the user is dragging/panning. Seeking to an event or showing a clip explicitly brings it into view.

### 9.3 Session identity

Some demos restart source timestamps. Keep `segment` on events, windows, lane keys, and selection targets throughout the redesign. Equal timestamps must not merge events from different sessions.

Density caches and marker bins must retain segment keys. If multiple sessions overlap at the same x coordinate, a density tooltip can show counts by session and a marker hit opens a small event chooser identifying session, actor, and time. Do not choose an arbitrary overlapping event or turn a density bucket into a synthetic event.

Do not manufacture a global concatenated timeline or bypass `starts_in`/export restrictions in this GUI task. Where the existing playback API cannot uniquely seek a later session by a repeated timestamp, show that ambiguity and disable the ambiguous jump/play action instead of silently displaying the first session. Events remain inspectable with session labels. Extending the underlying time model is separate work.

## 10. Inspector tabs

### 10.1 Events

Keep the tab header, filter controls, and bottom actions fixed; only the event list scrolls.

- First filter row: Kills, Shots, Switches, Actions, plus a coverage-information control.
- Second row: Weapon and “Jump to highlight…”. Wrap into separate rows when needed.
- While Kills is selected, a compact “Kills with” row: Midair, Speed, Wallgauss, and Clear when any refinement is on. Enabled options expand directly beneath it (§10.1.1).
- Event count: filtered count; optionally show total in a tooltip.
- List: time, actor, action/weapon, victim when present, HS/team-kill/suicide/environmental indicators where supported.
- Bottom: previous/next event, Play, Clip it.

Use a single predicate combining `Shown::kind`, `Shown::weapon`, and kill refinements uniformly for list rows, density, and timeline marks. Keep filtered row indices mapped into `Model.events`. Do not exempt actions from the weapon filter as the mockup does. For kill/death lane marks, evaluate the predicate on their shared event, not separately on the lane actor.

Always show the actor for shots, actions, and switches. Long actor/victim names truncate independently and expose full details on hover/click. Preserve `text::player_name` for stripped display names, raw-name details, unknown actors, event mode, and original weapon strings where classification is unknown.

Previous/next follows visible rows. If selected event is filtered out, disable Play/Clip it for that hidden selection and navigate relative to the playhead using the current `event_nav` rule. Do not silently select the first row. At boundaries disable the unavailable direction.

Single-click selects and seeks; double-click plays the clicked event with native padding. Process the clicked event explicitly before issuing Play so the previous selection is not played.

The highlight combo uses `Model.windows`: Golden minute is the first ranked window and the remaining choices are the existing best windows, with real times, score, and side counts. Selecting one shades its full interval, fits it with context, brings the first visible event in that interval into view, and seeks to its start when unambiguous. Do not treat a highlight as a single-event clip.

When a highlight is active, show a clear “Clip highlight” action, either adjacent to the combo or in the Clip it menu. It applies padding to the whole window and uses the same segment checks. Normal Clip it continues to mean the selected event. Creating either switches to Clips and focuses the new clip card.

Coverage explanations can be compact help. A real decode error or partial-analysis warning remains a visible badge/message with details; users must know why events stop before the demo ends.

#### 10.1.1 Kill-refinement state and controls

Add a GUI `kill_filters.rs` module containing state, pure matching functions, and badge formatting. Suggested shape:

```rust
enum MidairMode { Off, Either, Both, ExactlyOne }
enum SpeedSubject { Killer, Victim, Either }
struct KillFilters {
    midair: MidairMode,
    speed_enabled: bool,
    min_ups: u32,          // Initially 1000.
    speed_subject: SpeedSubject, // Initially Either.
    wallgauss: bool,
}
```

Default all three refinements off. Midair enables Either initially; display **Either / Both / Only one**, with tooltips explaining that Either includes both and Only one means exactly one. Speed reveals an “At least” `DragValue` with `ups` suffix and Killer / Victim / Either choices. Use nonnegative whole-number thresholds, drag increments around 50, and allow precise typing. Do not cap valid user input at the mockup's arbitrary 5000; reject nonfinite/negative/overflowing text without replacing the last valid value.

Enabled refinements combine with **AND on the same kill**. Each subject choice describes the test inside that refinement. Non-kill events still obey their kind/weapon toggles and are not removed by kill-only refinements. Turning Kills off hides the refinement form but preserves its values; turning it back on restores them. **Clear** resets refinements to off/default modes while retaining the entered `min_ups`; it does not clear kind toggles, weapon selection, library filters, clips, or playback state.

This state lives with the current demo's `Shown`, survives tab and pane changes, and initially resets for a new demo like existing event filters. Extend the filtered-row cache key with the whole state and a kill-enrichment revision. When classification completes, invalidate rows, markers, density, and badge summaries together. Do not cancel/restart an analysis job merely because the user changes a filter.

The form is in the Events tab only. It does not add implicit library SQL filters or change existing CLI `KillQuery` semantics. The current core `KillQuery` has killer/victim airborne booleans and an either-player speed threshold; that type alone cannot represent Exactly one or all new subject choices.

Keep event rows available by reserving the footer first and limiting the expanded filter area to a reasonable share of inspector height (about 45%); scroll the form if necessary. At 320-point inspector width wrap the speed subject choices beneath the number. Active refinements remain visible even if their options scroll.

#### 10.1.2 Existing motion data and its meaning

Reuse these existing fields:

- `Kill.context: Option<KillContext>` from `analysis/context.rs`.
- `KillContext.killer`, `.victim`, and `.shot` from `analysis/highlights.rs`.
- `MotionState.airborne`, `.horizontal_speed`, and `.evidence` from `analysis/motion.rs`.
- Existing cache `killer_airborne`, `victim_airborne`, `killer_speed`, and `victim_speed` columns already store optional motion information.

`Model::build` currently drops this kill context. Extend its kill-event presentation data with both optional airborne flags, both optional horizontal speeds, motion evidence, sampling-time/source metadata, and a stable kill key `(demo generation, segment, kill ordinal)`. Copy it while the report is available. Do not infer these values from displayed animation, jump height, current viewer frame, or end-of-match data.

For consistency with the existing core/library queries, **ups means horizontal speed**, `sqrt(vx² + vy²)`, not total 3D speed. Explain “Horizontal speed” in the filter help and badge tooltip. Compare unrounded finite values against `min_ups`; only round for display.

Current context measures the killer at the linked shot, or at the kill if no shot was linked; it measures the victim when the death message arrived. Preserve and disclose this timing, especially for projectile kills. “Both” means both sampled states are airborne, not a new assertion that they were simultaneously airborne at one shared instant. Retain `MotionEvidence::Exact`, `Trajectory`, `Unclear`, and `NotVisible`; identify estimates in details. Do not convert a missing linked shot into proof of a firing-time motion state.

Use the existing motion estimator initially. Its estimates are not server ground truth for other players. Unknown/out-of-view values remain absent; ambiguous movement such as a landing/stairs must not be repaired by assigning grounded. Reject invalid/nonfinite speed values independently of airborne availability: a usable speed can exist even when airborne classification is unclear.

#### 10.1.3 Matching with unknown values

The following rules deliberately differ from the mockup's JavaScript truthiness/default-zero behavior:

| Refinement | A kill matches when |
| --- | --- |
| Midair Either | At least one sampled airborne value is explicitly `Some(true)`; the other may be unknown. |
| Midair Both | Both values are explicitly `Some(true)`. |
| Midair Only one | One is explicitly true **and the other explicitly false**. True/unknown does not prove exactly one. |
| Speed Killer / Victim | That subject has a finite known horizontal speed ≥ the threshold. |
| Speed Either | At least one subject has a finite known horizontal speed ≥ the threshold. |
| Wallgauss | Gauss attribution plus a supported positive wallgauss reconstruction as defined below. |

Unknown does not match an active requirement it cannot establish, including a speed threshold of zero. It is not silently recorded as false or 0 ups. Pure matching should also report whether exclusion was due to insufficient evidence, so the UI can say “No matching known kills; some kills lack movement data” instead of implying none happened.

Implement matching as Match / NoMatch / Unknown internally. For AND, a known NoMatch wins, otherwise any Unknown makes the combined result Unknown. For Either, a known matching subject wins even if the other is unknown; if neither matches and at least one could be unknown, the result is Unknown. Include only Match in visible filtered rows; count Unknown separately without treating it as an error.

Wallgauss may coexist with a non-Gauss weapon filter. Keep both settings and show a clear empty result/conflicting-filter explanation; do not silently replace the user's weapon selection.

Timeline selection, previous/next navigation, Play, Clip it, and filtered counts operate on the same matched event IDs. Highlight rankings remain the existing full-demo analysis, not freshly ranked by these refinements; label the highlight menu/help accordingly. If a highlighted interval has no visible matching event, still show the interval and allow Clip highlight without choosing an unrelated event.

#### 10.1.4 Kill badges

At the trailing edge show known badges in this order: **HS**, **WALL**, **AIR/AIR ×2**, **speed**.

- HS retains current headshot semantics.
- WALL appears only for a positive classification, with tooltip “Wallgauss identified by reconstruction” and details of evidence, limitations, and direct-after-penetration or exit-blast route.
- AIR ×2 requires both known true. AIR means at least one known airborne; its tooltip names the known airborne player and explicitly says when the other state is unknown. AIR alone must not be interpreted as the Only one predicate.
- Display the highest **known** horizontal speed when it is at least 1000 ups or the Speed filter is enabled. The trigger uses the raw value, not rounded text. Tooltip lists both players' speeds separately, with Unknown for unavailable values and sampling/evidence descriptions. When the Speed filter is active but both speeds are unknown, use a muted `— ups` badge/detail rather than `0 ups`.

Give the killer and victim useful name space. At narrow widths use a second line for badges or a `+N` details control rather than shrinking names to nothing. If rows become two lines, use a consistent row height for that layout so virtualization and scroll-to-selected remain correct. Full details are available on click/keyboard, not only hover.

#### 10.1.5 Wallgauss analysis and evidence

**This is required new work, not a boolean already present in the demo analysis.** `gauss.sc` and a Gauss death notice do not by themselves distinguish a wall kill from a direct hit. The current `link_kills` chooses a likely preceding shot; it is a candidate association, not proof of the killing damage path.

Definition for this UI: a player-attributed Gauss kill whose supported reconstructed damage route depends on a successful wall penetration, including damage from the exit blast or a subsequent penetrating beam. A reflection-only kill, an ordinary direct hit before penetration, or shooting through another player alone is not wallgauss. If more than one plausible route remains, retain Unknown rather than prefer the wall route.

The server's `CGauss::Fire` applies damage as well as tracing: its successful penetration branch includes an exit blast, so requiring the final beam to intersect the victim would miss legitimate candidates. Primary/charged and multiplayer/singleplayer behavior differ. Use the applicable game/mod profile and validate with controlled recordings; neither client visuals nor a nearby wall alone establish causation. Reference: [Valve's server Gauss implementation](https://github.com/ValveSoftware/halflife/blob/master/dlls/gauss.cpp). Check mod differences against [OpenAG's Gauss source](https://github.com/YaLTeR/OpenAG/blob/master/dlls/gauss.cpp), without assuming it proves the behavior of every AG server.

Implement the following bounded pipeline:

1. During base analysis, collect each Gauss kill's identity, segment, server/source timing, original kill ordinal, candidate occurrence IDs, and relevant raw `RecordedEvent` arguments. Preserve `WeaponOccurrence.event`, including primary/charged flags, origin, aim, and energy. The existing linked shot can seed candidates; account for duplicate local/network records and same-time competing shots. A single shot can kill multiple victims.
2. Once the base model is published, start a **separate cancellable enrichment job** for the active demo when usable game assets are available. Base events and motion filters must appear immediately; wall analysis must not hold the base list hostage.
3. Reconstruct candidate shot-time source state in recording/group order using the existing CPU playback/decoder and `GameFiles` callbacks. Resolve each session's map and relevant brush/entity transforms, shooter eye/ducking, victim state, and data coverage. Do not trace against the user's current view or current loaded map. Bound memory and check cancellation between record groups.
4. Build a structured shot-path result with ordered impacts, successful entry/exit pairs, energy, player hits, and candidate damage regions. Reuse tested geometry primitives where valid. Keep a classifier separate from painter beams/glows; visual trails have neither enough identity nor sufficient proof of damage.
5. Correlate the candidate damage route with that kill's shooter/victim identity and timing. Require a supported penetration before that victim's candidate damage, sufficient source-state coverage, and no unresolved competing direct/reflection/other-shot explanation. Do not require knowledge of victim health merely to classify the observed death's route, but missing health/armor/hit geometry that prevents disambiguation must leave the route Unknown.
6. Publish results keyed by demo generation, segment, and kill ordinal, with a monotonically increasing enrichment revision. Ignore stale replies after demo or asset-root changes. Update filters/badges without seeking, changing selection, or restarting playback.

Use a type such as `WallGaussEvidence::{NotApplicable, Unknown(reason), ReconstructedNo(details), ReconstructedYes(details)}`; the displayed WALL badge and filter match only `ReconstructedYes`. Store the associated shot ID, route, map/profile fingerprint, and evidence limitations in details. “Reconstructed” must not be presented as an explicit server-recorded flag.

Known limitations require conservative handling:

- `lambda-slicer-playback/src/gauss.rs` implements client effects. Its own header notes a world-only penetration trace; `bullets.rs` uses approximate player boxes. Do not promote these approximations to authoritative kill classification.
- Missing or stale BSP/brush/model collision data, unsupported rotated/moving brush behavior, masked actors, invisible victims, packet gaps, unverified mod mechanics, and ambiguous candidate shots produce Unknown for affected kills.
- A confirmed direct hit followed by the beam penetrating a wall elsewhere must not mark that earlier victim as WALL.
- A successful penetration with no supported association to the recorded victim is not a positive kill classification.

Keep the dependency graph acyclic: put CPU classification and result types in `lambda-slicer-playback` (e.g. `kill_features.rs`), which already depends on core. GUI job code (e.g. `kill_analysis.rs`) supplies the existing asset callbacks. Core must not depend on playback, GUI, wgpu, or a game installation. Initially keep these asset-dependent results in memory for the opened demo; do not add a misleading unconditional wallgauss SQL column or force all library scans to load game assets.

Cache enrichment within the active-demo session by demo fingerprint, classifier/profile version, and resolved asset fingerprints. Invalidate on explicit asset-root changes and stale/mismatched map assets. Do not launch a fresh reconstruction for each slider movement or seek. If persisted enrichment is added later, these dependency fingerprints must be part of its key and Unknown must remain distinguishable from NotYetAnalysed.

With Wallgauss enabled, show **Checking wallgauss…**, an actual examined/total-candidate count if available, and partial-result status. Missing assets show “Wallgauss needs game files” with the same Locate action, while base events/clips and motion filters stay usable. Zero identified results plus unknown candidates is not “No wallgauss kills”. Offer Clear/disable so users can return to all kills.

G4a requires actual positive and negative controlled cases and real-demo inspection. A UI wired to a constant false/unknown value or to “charged Gauss” alone does not complete the feature. It is acceptable for unsupported/incompletely recorded cases to remain explicitly Unknown.

### 10.2 Clips

Top: New clip and Export all; while exporting show Stop exporting with current behavior. Middle: scrolling cards. Bottom: export destination shortcut opening Settings → Clips.

Each card contains:

1. Editable name and requested duration.
2. In/Out time fields, each with a Set to playhead icon.
3. Play, Loop, Show on timeline, Remove, and Export.
4. Compact effective-range information and any queue/export result.

Use the existing `DemoTime` parser and draft-edit behavior, including decimal comma and supported time formats. Do not clamp an invalid typed value into a different accepted range without telling the user. Retain the previous valid value, error outline/message, and editable draft.

“New clip” now starts at the **current viewer playhead**, matching its label, and extends ten seconds within the recording. Without a viewer use the current timeline start, then recording start as fallback. If the remaining range is empty, explain why it cannot export. This is an intentional change from the old “timeline start” behavior.

I/O shortcuts and Set to playhead affect only the explicitly chosen clip. If there is no chosen clip, disable them with a reason. Both update through the same validation path as manual editing; no implicit clip creation.

Keep all existing export semantics:

- Requested range and snapped/effective range are distinct.
- Unexportable clips show the concrete refusal; validation performed during export can still refuse a preliminary-valid clip.
- Export all queues eligible clips; make skipped ineligible clips visible.
- Commit a time-field blur edit before taking the export snapshot, including when the same click presses Export all.
- Freeze the queued job's source, range, and destination. Later edits must not relabel that job's result as an export of the edited range.
- Stop exporting cancels queued work after the current clip completes, matching existing behavior.
- Preserve temporary-file validation, no-overwrite publication, collision suffixes, safe filename generation, and source preservation.
- Show saved path, snapshot-validation result, and Copy `playdemo` command when valid for the mod folder; otherwise retain the copy-to-mod-folder instruction.
- Persistent clip-store load/save problems remain visible, and an unreadable store is never overwritten.

Only one clip loops at a time. Editing the looped clip moves its loop. Removing it clears the loop; removing an earlier card cannot transfer the loop to another clip. Starting event preview or a one-shot clip preview clears the prior loop as today.

A valid ordinary clip can have its rounding details behind a help control. A failure, invalid range, skipped export, or unsaved clip cannot be represented only by an invisible tooltip.

### 10.3 Scoreboard

Use the mockup's restrained typography, team sections, and numeric alignment while keeping `DemoView::scores` calculations.

Preserve:

- Final standings at the existing last-kill/fallback moment.
- Standings at the selected event, including its segment, with a visible “Show final” control.
- Authoritative team-score updates rather than summing player frags unconditionally.
- Teams, duels, free-for-all, one player, no scores, disconnected players, and negative scores.
- Recorder marker and player incarnation identity.
- Recorder accuracy; headshot coverage including the existing duel exception. Unknown values stay a dash.

The heading must state whether it shows final scores or a selected moment. Do not make continuous viewer playback silently change a final-score table. Keep the full selected-moment behavior even though the browser only shows final scores.

Use “Frags” or an accurately labeled abbreviation; the mockup's K label must not turn frag scores into inferred kill counts. A two-color comparison strip is optional only for exactly two comparable sides with nonnegative scores and a positive total. Hide it for negative/zero/other cases rather than dividing by an invalid sum.

Known red/blue teams may use the reference palette. Other teams and FFA players need stable, neutral/readable styling; do not infer team membership from a player's clothing color or a missing team field.

## 11. Settings window

Use a stable-ID `egui::Window`, initially centered, `.movable(true)`, noncollapsible, with an initial size around **600×460**. Remove the existing fixed `.anchor(...)`, which prevents free dragging. Allow sensible resizing for long translations. Constrain it to the current application content rectangle and recover an offscreen saved position.

The sidebar is initially 144 points. Its four sections remain visible; the content area scrolls independently. Fields use labels on the leading side and controls on the other side, with label width measured from localized text rather than always forcing 140 points. At small widths stack labels above controls. Switching sections retains entered values and per-section scroll state.

| Section | Fields and behavior |
| --- | --- |
| General | Half-Life asset folder plus Game executable Browse/Not set state and direct shared picker (§8.4); `--game` takes precedence and `None` does not imply automatic installation discovery. Language selection with all locales/system behavior and draft notice; analysis workers 1–4. |
| Library | Real scan folders, Add folder, Remove folder; effective cache path readout. Removing a scan root never deletes its files. Preserve existing scan/rescan behavior. |
| Viewer / Picture | Gamma 1.8–3, Brightness 0–3, Texture gamma 1.8–3, Light gamma 1.8–3, Overbright, and Game defaults affecting only those picture fields. |
| Viewer / View | Recorded FOV checkbox, manual FOV 10–150 when disabled, show viewmodel. |
| Viewer / See through walls | Players; projectiles/items sub-option disabled while Players is off without discarding its saved value. |
| Viewer / Sound | Master volume 0–1 with percent readout, synchronized with transport. |
| Clips / Export | Beside each source demo, or one configured folder; Browse to choose the actual folder. |
| Clips / Padding | Before and after, 0–120 seconds, native defaults 5 and 3 seconds. |

Use current `ViewerSettings::clamped`, `FOV`, and `MAX_PADDING` as authorities rather than duplicate numeric limits. Paths stay `PathBuf`; elision is display-only. Browse cancellation preserves the previous value.

Keep the system-language choice represented by `locale: None`, distinct from explicitly selecting the current system language. If the current picker lacks a visible “System language” item, add one. Show a launch-supplied game path as an override without writing it into saved settings.

Apply changes immediately with no artificial Save/Cancel dialog. General game-folder/executable changes use the reload path, preserve playback intent as specified in §8.4, and cancel old asset/enrichment work correctly. Volume and see-through do not reopen the viewer. Picture settings must not reset playback. Changing language updates the open window without losing focus-independent state.

Settings opened from picture/export/folder controls selects the corresponding section. The main gear reopens the last section visited during the session, initially General.

## 12. Keyboard, focus, and accessibility

Route keyboard actions in one place, then invoke the same semantic actions as buttons.

| Shortcut | Behavior |
| --- | --- |
| Ctrl+O | Open demo dialog |
| Ctrl+, | Open/close Settings |
| Space / L | Existing forward play/pause behavior |
| J | Existing reverse play/pause behavior |
| K | Pause |
| Comma / Period | Previous/next recorded group |
| Left / Right | Seek −/+5 seconds when not owned by a focused control |
| Home | Seek recording start |
| Ctrl+Left / Ctrl+Right | Previous/next visible event |
| I / O | Set chosen clip In/Out to current playhead |
| Escape | Cancel active drag/scrub first; otherwise dismiss popup/settings/drawer; close demo only when no closer interaction owns Escape |

Do not trigger letter shortcuts during text entry or IME composition. Focused sliders, combos, and buttons own their normal arrow/Space/Enter behavior. Suppress workspace playback shortcuts while Settings is the keyboard context. Consume handled keys so the timeline and viewer cannot both act.

Tabs and toggles use native selectable widgets; icon buttons retain keyboard activation and accessible names. Timeline painting needs focusable responses and meaningful value/description metadata, plus keyboard/button alternatives for zoom, Fit, seek, and clip ranges.

All new user-facing strings go through Fluent. Keep keys and argument shapes synchronized across the 14 catalogs and run existing localization checks. Preserve plural handling, localized number/date formatting, bidi names, pseudo-localization, and left-to-right time fields. Do not add English sentences directly to painter code.

## 13. Performance and implementation boundaries

The redesign must not add work proportional to the entire library on every frame.

- Table and compact list fetch only visible pages.
- Events use existing filtered-index caching and virtualized row painting.
- Density/marker caches invalidate on model/filter/enrichment-revision changes, not playhead movement.
- Team/actor display metadata is prepared from existing model data once or incrementally; do not run `standing_at` for every visible event every frame.
- Aggregate library statistics refresh alongside normal cache refreshes.
- Scorecard calculations should be cached by model generation and selected moment if extraction makes them expensive.
- GPU rendering continues through the existing callback. Do not read pixels back to display the viewer.
- UI painting does not parse demos, walk directories, export files, or wait for workers.
- Panel resize does not reload assets or recreate playback/audio workers.
- No polling loop should request continuous repaint while idle without actual pending work.
- Kill refinement changes run pure cached predicates; they do not reparse demos or launch geometry reconstruction. Wallgauss enrichment is cancellable and updates through stable IDs.
- Run at most one wallgauss enrichment job for the active demo, with bounded work slices; interactive playback/seek requests must remain responsive during it.

Do not alter the parser, weapon attribution, renderer shading, sound event simulation, or export safety rules to make a screenshot look like the mockup. Small data adapters such as reading existing `score_json`, cached map suggestions, and aggregate library counts are allowed. The new wallgauss classifier is the explicit exception to a GUI-only scope; implement it as evidence enrichment without rewriting existing kill attribution, scores, or export behavior.

## 14. Implementation stages and acceptance gates

Make each stage reviewable and keep the application running throughout. A stage is complete only after its listed behavior works with real demos.

### G0 — Baseline and separation

1. Read current `docs/gui.md`, `gui-text.md`, and the relevant code listed in §2.
2. Record the starting commit and inspect uncommitted work.
3. Capture current library, open-demo, settings, and export-result screenshots at 1280×800.
4. Extract viewer frame processing from viewport/transport painting and demo background work from tab bodies.
5. Add shared presentation state and semantic action methods without changing the visible workflow yet.
6. Define kill-feature presentation types and stable kill identities, preserving existing motion context before the report is dropped.

**Gate:** Existing playback, reverse, stepping, event preview, clip looping/editing, export, capture, and scan completion still work. No duplicate polling/audio feeds or dropped settings changes.

### G1 — Theme and reusable widgets

1. Add the palette, spacing, type sizes, icon buttons, status indicator, compact score presentation, and help-popover widget.
2. Apply the theme to the current UI.
3. Add the new localization keys and reusable layout helpers.

**Gate:** Text remains readable in English, Arabic, and pseudo-localization; current app icon/branding remains intact; keyboard focus and disabled-state reasons are visible.

### G2 — Library modes

1. Split filters, table, compact list, and query preparation.
2. Implement grouped filter pane and Columns menu with old column-key migration.
3. Add structured score reading, Score sort, and cached total/intensity aggregates.
4. Add compact rows and the shared sort/search/filter state.
5. Wire top toolbar/status strip and platform path actions.

**Gate:** Hundreds/thousands of rows remain paged; table/list return identical query results and sort order; direct file opening works without a cache; no hidden columns become inaccessible.

### G3 — Workspace and transport

1. Replace the existing 42%-width library split with the new pane hierarchy.
2. Add header, inspector tabs, collapsible list, and responsive layouts.
3. Move the existing real viewport into the middle.
4. Replace transport text clutter with labeled icon controls, keeping reverse and actual group stepping.
5. Keep inspector methods functional even before their final visual polish.
6. Implement the direct installation picker, validation states, legacy-folder handling, and asset-free controller mode from §8.4. Wire both Locate and Settings Browse to the same action.

**Gate:** Switching tabs and resizing preserves the same viewer/controller, time, sound, selected event, and clips. Missing game folder/map failure does not block analysis or export. In setup mode transport/timeline still work; selecting a valid installation restores picture/sound at the retained position without losing clips or filters.

### G4 — Timeline and Events

1. Add the compact track, cached density, overview window, and zoom/Fit controls.
2. Preserve detailed lanes behind Player lanes.
3. Implement one hit-test/drag state machine and the gesture contract.
4. Restyle Events and wire shared selection, navigation, preview, highlight, and Clip it actions.
5. Remove the redundant old seek slider/old timeline visibility checkbox.
6. Add Midair/Speed controls, pure tri-state predicates, motion badges, and the common filter/cache invalidation path. Expose Wallgauss with truthful pending/unavailable states until G4a is implemented; do not claim that feature complete yet.

**Gate:** Marker click, event click, scrubbing, and viewer time agree; panning never seeks; event filters agree everywhere; repeated-session timestamps do not silently seek the wrong recording segment.

### G4a — Kill features and wallgauss enrichment

1. Complete forwarding of both players' movement values, evidence, sampling times, team identities, and stable kill keys into the GUI model.
2. Test the Midair truth table and Killer/Victim/Either speed predicates, including unknown values and exact threshold boundaries.
3. Implement candidate extraction and the separate cancellable enrichment worker in §10.1.5. Verify the chosen game/mod mechanics against primary source and controlled recordings.
4. Add structured trace/damage-route results, conservative classifications, revision handling, and missing-asset/pending/unknown feedback.
5. Connect WALL badges and the Wallgauss toggle to real results. Ensure filters, timeline death/kill marks, density, navigation, and counts agree after asynchronous updates.
6. Record at least one supported through-wall kill, an exit-blast case, a direct Gauss kill, a reflection-only case, and an incomplete-evidence case.

**Gate:** All three refinements work on real analysis, combined filters match the same kill, and Unknown is never turned into grounded/0 ups/not-wallgauss. Wallgauss positives are supported by the reconstructed route and controlled validation. Missing or ambiguous evidence remains explained rather than guessed. Neither a dummy classifier nor the presence of a charged shot satisfies this gate.

### G5 — Clips and Scoreboard

1. Convert clips to cards with stable runtime IDs, In/Out playhead actions, and preserved draft commits.
2. Move real export queue/results into the cards and fixed footer.
3. Retain loops, effective bounds, no-overwrite exports, and persistence notices.
4. Restyle the existing standings in the Scoreboard tab.

**Gate:** The same valid ranges export through the unchanged export path; invalid/ambiguous ranges still refuse with a reason. Editing then clicking Export uses the new bounds. Scoreboard handles teams, FFA, duels, unknown statistics, and selected/final moments correctly.

### G6 — Settings and persistence

1. Implement the movable four-section window and direct-section shortcuts.
2. Migrate legacy timeline visibility and add bounded presentation preferences.
3. Retain native limits/defaults, game-folder reload behavior, launch overrides, and clip-store format.
4. Complete keyboard/focus routing and all locale catalogs.
5. Persist the optional executable locator and synchronize it with folder changes; verify legacy files without the new field still render from a valid asset folder.

**Gate:** Existing settings load without losing fields; settings survive restart; legacy timeline preference expands lanes correctly; screenshot runs do not modify personal preferences.

### G7 — Validation and documentation

1. Run the targeted regression checks and manual workflows below.
2. Fix visual clipping and interaction regressions at all specified sizes.
3. Run the repository checks on Windows and, where available, `scripts/check.ps1 -Linux`.
4. Update `docs/gui.md` and screenshots for the actual final behavior. Update CLI help for `--timeline` and any explicit screenshot-navigation flags.
5. Add a short validation record with commit, commands, demo paths, tested sizes/locales, and outstanding limitations.

**Gate:** Deliver working native UI, not a mock-data screenshot. Do not mark Linux GUI interaction or audio-device testing complete based only on a cross-platform compile.

## 15. Verification plan

### 15.1 Focused automated checks

Reuse existing tests; add tests for changed behavior that can regress:

1. Old settings JSON migrates without losing locale, folders, hidden columns, gamma, FOV, volume, or clip padding. Missing new values and unknown enum values fall back safely.
2. Score display correctly handles 0/1/2/many sides, negative scores, missing JSON, and malformed JSON. Score sort is correct across page boundaries and ties.
3. Filter counts and table/list query bindings agree, including whitespace, inclusive dates, and majority-spectating thresholds.
4. Timeline coordinate transforms/clamps round-trip at zoom boundaries, tiny durations, empty models, and high DPI; gestures resolve clip handle vs scrub vs event vs pan deterministically.
5. Density counts unique filtered events, not duplicate death marks, and does not merge session identities.
6. Event navigation works from no selection, a filtered-out selection, and first/last rows. Double-click previews the clicked event.
7. Same-frame time commit followed by Export uses committed bounds; removing/editing clips preserves correct selection and loop ownership.
8. Switching inspector tabs never pauses/polls playback twice or discards export completion; newer demo loads cancel stale seek work.
9. Existing localization key/argument, bidi, and plural tests pass with the new strings.
10. Kill features survive `AnalysisReport` → `Model::build`; record/segment/player-incarnation identity and sampling-time semantics are preserved.
11. Midair tests cover all nine True/False/Unknown pairs for Either, Both, and Only one. True/Unknown matches Either, not Both or Only one.
12. Speed tests cover killer/victim/either, zero threshold with unknown values, 999.9/1000/1000.1 ups, nonfinite values, large typed thresholds, and rounding that must not affect matching.
13. Combined refinements use the same kill, do not remove enabled non-kill event kinds, and are shared by event rows, kill/death markers, density, and navigation. Clear preserves ups; hiding Kills preserves refinement choices; pending enrichment invalidates all dependent caches once its revision changes.
14. Installation picker tests cover Cancel, wrong filename, missing file, valid root, asset-only root, migrated folder-only settings, mismatched executable/folder, launch override, and Linux without `.exe`. Validation failure never overwrites a working choice.
15. Asset-free transport and subsequent setup preserve source time, rate, pause/direction, loop/preview limits, clips, and selection while rejecting stale worker replies.
16. Wallgauss tests cover penetration before victim damage, exit-blast damage, direct hit followed by unrelated wall penetration, charged shot with no penetration, reflection-only damage, more than one victim, duplicate event evidence, competing shots, reused player slots/sessions, missing maps, unsupported geometry, and invisible victim state. Verify cancellation and asset-fingerprint invalidation.

Do not add tests that merely assert the exact RGB constants or duplicate widget implementation. Use screenshots and interaction checks for visual quality.

From the repository root, use targeted package tests during the relevant stage, then the existing final check script:

```powershell
cargo test -p lambda-slicer-gui --locked
cargo test -p lambda-slicer-core --test library --locked
.\scripts\check.ps1 -Linux
```

The final script runs formatting, workspace clippy with warnings denied, tests, and a release build on Windows, then Linux checks through WSL. If WSL is unavailable, run `.\scripts\check.ps1` and record Linux as unverified; do not silently omit it from the validation record.

### 15.2 Manual workflows with real data

Use existing demos under `F:\HLScreeniesDemos` and `F:\SteamLibrary\steamapps\common\Half-Life\ag`. Use the game at `F:\SteamLibrary\steamapps\common\Half-Life`. Resolve actual demo paths before scripting; do not hardcode a guessed parent folder.

Representative demos from existing project work include `run_92.dem`, `naz_crossfire_pb_01.dem`, `stlakfa.dem`, and `stalkx_5o5_asasasdgwgw.dem`. Cover a duel, a team match, and a single-player/no-events or equivalent supported fixture. Keep unusual/unsupported examples for the error states.

Check:

- Open through dialog and drag/drop; add a folder; scan, cancel, and see final refreshed counts.
- Filter Gauss and player, change sorting, open a demo, use compact filters, hide/show list, close it, and confirm query state is retained.
- Play with sound; reverse; step actual groups; change speed; rapidly seek; pause/resume; switch demos during a cold seek.
- Use Events and Clips while playback continues; ensure selecting a row seeks and merely changing tabs does not.
- Inspect charge/beam spans in player lanes and actor identity on shot/action rows.
- Jump to a highlight, clip the full highlight, then create a separate event clip.
- Edit In/Out and immediately click Export; inspect effective bounds; play/loop; change loop bounds; remove an earlier clip and then the looped clip.
- Export a valid clip, verify the existing validation result, and play the new file in Half-Life. Confirm the original is untouched. Check a refused range and a destination error.
- Change all settings sections, drag the window, shrink the main window, restart, and verify persistence.
- Remove/unset the game folder, use a missing-map demo, inspect partial analysis, and verify events/clipping remain available.
- Exercise keyboard focus in Search, clip name, time fields, sliders, combos, Settings, and IME text entry; Space/I/O must not leak into playback while typing.
- Start with neither installation nor executable configured. Use Locate, cancel it, choose an invalid file, then choose the real `hl.exe`; verify inline feedback, saved paths, and immediate viewer recovery. Repeat via Settings Browse and with a pre-existing folder-only configuration.
- While setup is displayed, use Play/Pause, source stepping, timeline seeking, event selection, and In/Out editing. Confirm no picture/sound is fabricated, then configure assets and check position/loop continuity.
- Test every Midair mode, all speed subjects at 1000 and another threshold, Wallgauss alone, and combined refinements. Check the AIR ×2/AIR/ups/WALL tooltips and confirm that clearing preserves the threshold.
- Enable Shots and Kills together with a kill refinement; confirm unrelated enabled shots remain visible, but filtered-out kills disappear from both killer and victim lanes.
- Enable Wallgauss without assets and during enrichment, change demos while it runs, then verify pending/unknown messages and that old results never enter the new demo.
- Compare labelled wallgauss cases with controlled in-game recordings. Include a real positive; a collection of Unknown cases is not evidence the classifier works. Record profile/map versions and remaining unsupported cases.

### 15.3 Visual acceptance matrix

Capture the following at **1280×800** and **1920×1080**:

- Library with filters open, populated table, and Columns menu.
- Demo workspace with Events, selected event, real viewer frame, and compact timeline.
- Clips with valid, invalid, exporting, and saved-result cards.
- Scoreboard for a team match and an FFA/duel.
- Each of the four Settings sections.
- Missing-installation warning and General's Not set/Browse state, plus invalid-path feedback.
- Events with all kill refinements expanded, unknown-data feedback, and rows containing HS/WALL/AIR/speed badges.

Also inspect **1024×768**, **900×600**, 150%/200% display scaling, Arabic, and the existing pseudo-locale. Long filenames, names, paths, error text, and dates must not cover neighboring controls or push export/navigation buttons out of reach.

Use the existing CLI screenshot options (`--size`, `--locale`, `--pseudo`, `--settings`, `--demo`, `--game`, `--viewer-at`, `--screenshot`) with isolated cache/clip fixtures. Screenshot mode already disables settings writes; do not test by modifying the user's real clip store. If repeatable inspector/settings-section captures need new CLI options, add explicit flags that only select presentation state and preserve the existing capture behavior.

Compare the layout and hierarchy with the supplied live/local reference, not the sample player names or static image. Native text rasterization need not be pixel-identical to the browser. The pane proportions, dark palette, compact spacing, clear selection, fixed action areas, and reduced visual noise should be recognizably the same design.

### 15.4 Performance acceptance

Use the same machine, release build, demo, viewer resolution, and settings before/after. Record library scroll behavior, event-list scrolling, idle CPU, cold-open time, seek responsiveness, and the existing viewer benchmark. Investigate a repeatable frame-time regression above roughly 10% before accepting the redesign; do not compare a newly larger viewport against a smaller baseline viewport.

The visible UI must remain responsive during scans, cold seeks, and exports. Counts must be real; failures must remain understandable; all requested controls must invoke actual application behavior.

## 16. Completion checklist

- [ ] Full-table library and compact demo list share one real query and sort.
- [ ] All original columns and filters remain accessible.
- [ ] Locate hl.exe and Settings Browse use a real shared picker, with valid migration and Linux alternatives.
- [ ] Setup mode retains working native transport/timeline/clip controls and restores viewing without resetting the workflow.
- [ ] Real playback, sound, reverse, group stepping, and countdown survive the layout refactor.
- [ ] Compact timeline is always available; detailed player/weapon lanes remain accessible.
- [ ] Events, timeline, highlights, selected clips, and viewer use shared state.
- [ ] Midair, configurable killer/victim/either speed, and Wallgauss refinements use real evidence and consistent unknown-value rules.
- [ ] Kill badges and tooltips identify subjects, horizontal-speed semantics, and reconstructed wallgauss evidence accurately.
- [ ] G4a passes with supported positive/negative wallgauss cases, including exit-blast and incomplete-evidence cases.
- [ ] Clips persist and export through the original validated, non-overwriting path.
- [ ] Scoreboard preserves actual scoring and coverage semantics.
- [ ] All settings work in the four-section movable window and retain existing saved values.
- [ ] Keyboard focus, RTL, localization, and small-window layouts pass.
- [ ] Windows checks and available Linux checks pass; untested behavior is identified accurately.
- [ ] Documentation and screenshots describe the implemented result rather than the React mockup.
