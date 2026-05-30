use mlua::prelude::*;
use mlua::UserData;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// TODO:
// - Syntax highlighting (evil)
// - Fix FL Studio ignoring inputs (other daws can type in the code box just fine??)
// - Align code to top of box
// - Make text line (i forget what its called, the | thingy that shows where you are editing the text) not really hard to see
// - Fix saving of lua text
// - Fix saving of advanced mode
// - Presets and presets system
// - make the ui less weird? the buttons kinda go off the bottom (though i kinda like that look?) and the simple/advanced mode thing is kind of weird (maybe figure it out from the script itself)
// - Indication for if the script hasn been ran/compiled yet.
// - Reset button that sets it back to the template script (though that can just be built into presets)
// - fix help text at the top (the one that shows what function you are supposed to use)
// - seperate into multiple scripts (this one is getting really long :/ )
// - fix editor snapping back down to the bottom if you click something
// - fix not being able to see the top of the script if it gets too long
// - Panic button
// - Infinite loop defence

#[derive(Clone)]
struct AudioContext {
    sample_rate: f32,
    tempo: f32,
    is_playing: bool,
}

impl UserData for AudioContext {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("sample_rate", |_, this| Ok(this.sample_rate));
        fields.add_field_method_get("tempo", |_, this| Ok(this.tempo));
        fields.add_field_method_get("is_playing", |_, this| Ok(this.is_playing));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(_: &mut M) {}

    fn register(registry: &mut LuaUserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}

#[derive(Clone)]
struct DspAPI {
    buffers: Arc<Mutex<HashMap<String, (Vec<f32>, usize)>>>,
}

impl DspAPI {
    fn new() -> Self {
        Self {
            buffers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl UserData for DspAPI {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "delay_read",
            |_, this, (name, delay_samples): (String, usize)| {
                let mut map = this.buffers.lock().unwrap();
                let (buffer, write_idx) = map
                    .entry(name)
                    .or_insert_with(|| (vec![0.0; 44100 * 10], 0));

                let len = buffer.len();
                let mut read_idx = (*write_idx as isize) - (delay_samples as isize);
                while read_idx < 0 {
                    read_idx += len as isize;
                }

                Ok(buffer[read_idx as usize % len])
            },
        );

        methods.add_method("delay_write", |_, this, (name, value): (String, f32)| {
            let mut map = this.buffers.lock().unwrap();
            let (buffer, write_idx) = map
                .entry(name)
                .or_insert_with(|| (vec![0.0; 44100 * 10], 0));

            buffer[*write_idx] = value;
            *write_idx = (*write_idx + 1) % buffer.len();
            Ok(())
        });
    }

    fn add_fields<F: LuaUserDataFields<Self>>(_: &mut F) {}

    fn register(registry: &mut LuaUserDataRegistry<Self>) {
        Self::add_fields(registry);
        Self::add_methods(registry);
    }
}

struct LuaEngine {
    lua: Lua,
    dsp: DspAPI,
}

impl LuaEngine {
    fn new() -> LuaResult<Self> {
        Ok(Self {
            lua: Lua::new(),
            dsp: DspAPI::new(),
        })
    }

    fn load_script(&self, code: &str) -> LuaResult<()> {
        self.lua.load(code).exec()
    }

    fn process_sample(&self, left: f32, right: f32) -> LuaResult<(f32, f32)> {
        let process: LuaFunction = self.lua.globals().get("process")?;
        let (l, r): (f32, f32) = process.call((left, right))?;
        Ok((l, r))
    }

    fn process_block(
        &self,
        left: Vec<f32>,
        right: Vec<f32>,
        ctx: AudioContext,
    ) -> LuaResult<(Vec<f32>, Vec<f32>)> {
        let process_block: LuaFunction = self.lua.globals().get("process_block")?;
        let (l, r): (Vec<f32>, Vec<f32>) =
            process_block.call((left, right, ctx, self.dsp.clone()))?;
        Ok((l, r))
    }
}

struct LuaSound {
    params: Arc<LuaSoundParams>,
    engine: Arc<Mutex<Option<LuaEngine>>>,
}

#[derive(Params)]
struct LuaSoundParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[persist = "lua-script"]
    pub script: Arc<Mutex<String>>,

    #[persist = "is-advanced-mode"]
    pub is_advanced: Arc<AtomicBool>,
}

const DEFAULT_SCRIPT: &str = r#"-- Simple Mode
function process(left, right)
    return left, right
end"#;

const ADVANCED_SCRIPT: &str = r#"-- Advanced Mode
-- block sizes vary (e.g. 128, 512 samples)
function process_block(left, right, ctx, dsp)
    -- example: pass through
    return left, right
end"#;

impl Default for LuaSound {
    fn default() -> Self {
        Self {
            params: Arc::new(LuaSoundParams::default()),
            engine: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for LuaSoundParams {
    fn default() -> Self {
        Self {
            editor_state: ViziaState::new(|| (650, 450)),
            script: Arc::new(Mutex::new(DEFAULT_SCRIPT.to_string())),
            is_advanced: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Debug)]
enum EditorEvent {
    SetScript(String),
    Apply,
    Import,
    Export,
    Imported(String),
    Exported,
    ToggleMode,
}

#[derive(Lens, Clone)]
struct EditorData {
    script: String,
    status: String,
    status_ok: bool,
    is_advanced: bool,
    #[lens(ignore)]
    engine: Arc<Mutex<Option<LuaEngine>>>,
    #[lens(ignore)]
    script_store: Arc<Mutex<String>>,
    #[lens(ignore)]
    is_advanced_store: Arc<AtomicBool>,
}

impl Model for EditorData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e: &EditorEvent, _| match e {
            EditorEvent::SetScript(s) => {
                self.script = s.clone();
            }

            EditorEvent::Apply => {
                *self.script_store.lock().unwrap() = self.script.clone();

                let result = LuaEngine::new().and_then(|eng| {
                    eng.load_script(&self.script)?;
                    Ok(eng)
                });

                match result {
                    Ok(eng) => {
                        *self.engine.lock().unwrap() = Some(eng);
                        self.status = "Script loaded".to_string();
                        self.status_ok = true;
                    }
                    Err(e) => {
                        *self.engine.lock().unwrap() = None;
                        self.status = format!("Error: {}", e);
                        self.status_ok = false;
                    }
                }
            }

            EditorEvent::ToggleMode => {
                self.is_advanced = !self.is_advanced;
                self.is_advanced_store
                    .store(self.is_advanced, Ordering::Relaxed);

                if self.script.trim() == DEFAULT_SCRIPT
                    || self.script.trim() == ADVANCED_SCRIPT
                    || self.script.is_empty()
                {
                    self.script = if self.is_advanced {
                        ADVANCED_SCRIPT.to_string()
                    } else {
                        DEFAULT_SCRIPT.to_string()
                    };
                }

                self.status = if self.is_advanced {
                    "Switched to Advanced Mode".to_string()
                } else {
                    "Switched to Simple Mode".to_string()
                };
                self.status_ok = true;
            }

            EditorEvent::Import => {
                _cx.spawn(|cx| {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Lua Script", &["lua", "txt"])
                        .pick_file()
                    {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let _ = cx.emit(EditorEvent::Imported(content));
                        }
                    }
                });
            }

            EditorEvent::Imported(content) => {
                self.script = content.clone();
                self.status = "Script imported. Press Run.".to_string();
                self.status_ok = true;
            }

            EditorEvent::Export => {
                let script_copy = self.script.clone();

                _cx.spawn(|cx| {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Lua Script", &["lua"])
                        .set_file_name("effect.lua")
                        .save_file()
                    {
                        if std::fs::write(path, script_copy).is_ok() {
                            let _ = cx.emit(EditorEvent::Exported);
                        }
                    }
                });
            }

            EditorEvent::Exported => {
                self.status = "Script exported".to_string();
                self.status_ok = true;
            }
        });
    }
}

impl Plugin for LuaSound {
    const NAME: &'static str = "Lua Sound";
    const VENDOR: &'static str = "DogeisCut";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = "1.0.0";

    type SysExMessage = ();
    type BackgroundTask = ();

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let script = self.params.script.lock().unwrap().clone();
        if let Ok(eng) = LuaEngine::new() {
            if eng.load_script(&script).is_ok() {
                *self.engine.lock().unwrap() = Some(eng);
            }
        }
        true
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let engine = self.engine.clone();
        let script_store = self.params.script.clone();
        let is_advanced_store = self.params.is_advanced.clone();

        let initial_script = script_store.lock().unwrap().clone();
        let initial_mode = is_advanced_store.load(Ordering::Relaxed);

        create_vizia_editor(
            self.params.editor_state.clone(),
            ViziaTheming::Custom,
            move |cx, _| {
                EditorData {
                    script: initial_script.clone(),
                    status: "Press Run to activate script".to_string(),
                    status_ok: true,
                    is_advanced: initial_mode,
                    engine: engine.clone(),
                    script_store: script_store.clone(),
                    is_advanced_store: is_advanced_store.clone(),
                }
                .build(cx);

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Lua Sound")
                            .color(Color::rgb(190, 160, 255))
                            .font_size(13.0)
                            .left(Pixels(10.0))
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0));

                        //let advanced = is_adv.get(cx);
                        Label::new(
                            cx,
                            // if advanced {
                            //     "function process(left, right) -> (left, right)"
                            // } else {
                            //     "function process_block(left, right, ctx, dsp) -> (left, right)"
                            // },
                            "",
                        )
                        .color(Color::rgba(140, 120, 180, 180))
                        .font_size(11.0)
                        .left(Pixels(12.0))
                        .top(Stretch(1.0))
                        .bottom(Stretch(1.0));
                    })
                    .height(Pixels(28.0))
                    .background_color(Color::rgb(18, 10, 32))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .border_radius(Pixels(0.0));

                    Textbox::new_multiline(cx, EditorData::script, false)
                        .on_edit(|cx, text| cx.emit(EditorEvent::SetScript(text)))
                        .font_family(vec![
                            FamilyOwned::Name("Consolas".to_string()),
                            FamilyOwned::Name("Courier New".to_string()),
                            FamilyOwned::Monospace,
                        ])
                        .font_size(13.0)
                        .color(Color::rgb(210, 205, 235))
                        .background_color(Color::rgb(13, 10, 22))
                        .border_width(Pixels(0.0))
                        .width(Stretch(1.0))
                        .height(Stretch(1.0))
                        .border_radius(Pixels(0.0));

                    HStack::new(cx, |cx| {
                        Binding::new(cx, EditorData::status_ok, |cx, is_ok| {
                            let ok = is_ok.get(cx);
                            Label::new(cx, EditorData::status)
                                .color(if ok {
                                    Color::rgb(100, 210, 130)
                                } else {
                                    Color::rgb(230, 90, 90)
                                })
                                .font_size(11.0)
                                .width(Stretch(1.0))
                                .left(Pixels(8.0));
                        });

                        Binding::new(cx, EditorData::is_advanced, |cx, is_adv| {
                            let advanced = is_adv.get(cx);
                            Button::new(
                                cx,
                                |ex| ex.emit(EditorEvent::ToggleMode),
                                move |cx| {
                                    Label::new(
                                        cx,
                                        if advanced { "Mode: Adv" } else { "Mode: Simp" },
                                    )
                                },
                            )
                            .background_color(if advanced {
                                Color::rgb(120, 50, 50)
                            } else {
                                Color::rgb(40, 80, 120)
                            })
                            .color(Color::rgb(220, 220, 220))
                            .border_radius(Pixels(3.0))
                            .right(Pixels(6.0))
                            .top(Pixels(4.0))
                            .bottom(Pixels(4.0));
                        });

                        Button::new(
                            cx,
                            |ex| ex.emit(EditorEvent::Apply),
                            |cx| Label::new(cx, "Run"),
                        )
                        .background_color(Color::rgb(70, 35, 120))
                        .color(Color::rgb(220, 200, 255))
                        .border_radius(Pixels(3.0))
                        .right(Pixels(6.0))
                        .top(Pixels(4.0))
                        .bottom(Pixels(4.0));

                        Button::new(
                            cx,
                            |ex| ex.emit(EditorEvent::Export),
                            |cx| Label::new(cx, "Export"),
                        )
                        .background_color(Color::rgb(40, 45, 60))
                        .color(Color::rgb(200, 200, 200))
                        .border_radius(Pixels(3.0))
                        .right(Pixels(6.0))
                        .top(Pixels(4.0))
                        .bottom(Pixels(4.0));

                        Button::new(
                            cx,
                            |ex| ex.emit(EditorEvent::Import),
                            |cx| Label::new(cx, "Import"),
                        )
                        .background_color(Color::rgb(40, 45, 60))
                        .color(Color::rgb(200, 200, 200))
                        .border_radius(Pixels(3.0))
                        .right(Pixels(6.0))
                        .top(Pixels(4.0))
                        .bottom(Pixels(4.0));
                    })
                    .height(Pixels(32.0))
                    .background_color(Color::rgb(10, 8, 20))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0));
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0));
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let is_advanced = self.params.is_advanced.load(Ordering::Relaxed);

        if let Ok(guard) = self.engine.try_lock() {
            if let Some(engine) = guard.as_ref() {
                if is_advanced {
                    let sample_rate = context.transport().sample_rate;
                    let tempo = context.transport().tempo.unwrap_or(120.0) as f32;
                    let is_playing = context.transport().playing;

                    let ctx = AudioContext {
                        sample_rate,
                        tempo,
                        is_playing,
                    };

                    let mut left_vec = Vec::with_capacity(buffer.samples());
                    let mut right_vec = Vec::with_capacity(buffer.samples());

                    for mut channel_samples in buffer.iter_samples() {
                        left_vec.push(channel_samples.get_mut(0).copied().unwrap_or(0.0));
                        right_vec.push(channel_samples.get_mut(1).copied().unwrap_or(0.0));
                    }

                    if let Ok((new_l, new_r)) = engine.process_block(left_vec, right_vec, ctx) {
                        let mut i = 0;
                        for mut channel_samples in buffer.iter_samples() {
                            if let Some(l) = channel_samples.get_mut(0) {
                                *l = *new_l.get(i).unwrap_or(&0.0);
                            }
                            if let Some(r) = channel_samples.get_mut(1) {
                                *r = *new_r.get(i).unwrap_or(&0.0);
                            }
                            i += 1;
                        }
                    }
                } else {
                    for mut channel_samples in buffer.iter_samples() {
                        let left = channel_samples.get_mut(0).copied().unwrap_or(0.0);
                        let right = channel_samples.get_mut(1).copied().unwrap_or(0.0);

                        if let Ok((new_l, new_r)) = engine.process_sample(left, right) {
                            if let Some(l) = channel_samples.get_mut(0) {
                                *l = new_l;
                            }
                            if let Some(r) = channel_samples.get_mut(1) {
                                *r = new_r;
                            }
                        }
                    }
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for LuaSound {
    const CLAP_ID: &'static str = "com.dogeiscut.luasound";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Write Lua expressions to make your own effects!");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::MultiEffects, ClapFeature::Utility];
}

impl Vst3Plugin for LuaSound {
    const VST3_CLASS_ID: [u8; 16] = *b"DogeisLuaSound67";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(LuaSound);
nih_export_vst3!(LuaSound);
