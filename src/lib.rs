mod editor;
mod engine;
mod presets;

use editor::{EditorData, EditorEvent};
use engine::{AudioContext, LuaEngine};
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub const DEFAULT_SCRIPT: &str = r#"function process_block(left, right, ctx, dsp)
    for i = 1, #left do
        -- You can manipulate individual samples here
        -- left[i] = left[i] * 0.5 
        -- right[i] = right[i] * 0.5
    end

    -- example: pass through
    return left, right
end"#;

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

        create_vizia_editor(
            self.params.editor_state.clone(),
            ViziaTheming::Custom,
            move |cx, _| {
                let current_script = script_store.lock().unwrap().clone();

                EditorData {
                    script: current_script,
                    presets: presets::load_presets(),
                    status: "Press Run to activate script".to_string(),
                    status_ok: true,
                    is_dirty: true,
                    is_preset_menu_open: false,
                    pending_save: false,
                    save_name: String::new(),
                    pending_delete: None,
                    engine: engine.clone(),
                    script_store: script_store.clone(),
                }
                .build(cx);
                ZStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        HStack::new(cx, |cx| {
                            Label::new(cx, "Lua Sound")
                                .color(Color::rgb(190, 160, 255))
                                .font_size(13.0)
                                .left(Pixels(10.0))
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0));

                            Label::new(
                                cx,
                                "function process_block(left, right, ctx, dsp) -> (left, right)",
                            )
                            .color(Color::rgba(140, 120, 180, 180))
                            .font_size(11.0)
                            .left(Pixels(12.0))
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0));

                            Button::new(
                                cx,
                                |cx| cx.emit(EditorEvent::TogglePresetMenu),
                                |cx| Label::new(cx, "Presets"),
                            )
                            .background_color(Color::rgb(40, 45, 60))
                            .color(Color::rgb(200, 200, 200))
                            .border_radius(Pixels(3.0))
                            .left(Stretch(1.0))
                            .right(Pixels(10.0))
                            .top(Pixels(4.0))
                            .bottom(Pixels(4.0));
                        })
                        .height(Pixels(28.0))
                        .background_color(Color::rgb(18, 10, 32))
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .border_radius(Pixels(0.0));

                        Textbox::new_multiline(cx, EditorData::script, false)
                            .on_edit(|cx, text| cx.emit(EditorEvent::SetScript(text)))
                            .on_mouse_down(|cx, _| {
                                cx.focus();
                                cx.capture();
                            })
                            .font_family(vec![
                                FamilyOwned::Name("Consolas".to_string()),
                                FamilyOwned::Name("Courier New".to_string()),
                                FamilyOwned::Monospace,
                            ])
                            .caret_color(Color::rgb(190, 160, 255))
                            .selection_color(Color::rgba(120, 80, 200, 120))
                            .font_size(13.0)
                            .color(Color::rgb(210, 205, 235))
                            .background_color(Color::rgb(13, 10, 22))
                            .border_width(Pixels(0.0))
                            .width(Stretch(1.0))
                            .height(Stretch(1.0))
                            .child_top(Pixels(0.0))
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

                            Binding::new(cx, EditorData::is_dirty, |cx, is_dirty| {
                                let dirty = is_dirty.get(cx);
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
                                .bottom(Pixels(4.0))
                                .disabled(!dirty);
                            });

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

                    Binding::new(cx, EditorData::is_preset_menu_open, |cx, is_open| {
                        if is_open.get(cx) {
                            VStack::new(cx, |cx| {
                                Button::new(
                                    cx,
                                    |cx| cx.emit(EditorEvent::InitiateSave),
                                    |cx| Label::new(cx, "Save Current"),
                                )
                                .background_color(Color::rgb(40, 45, 60))
                                .color(Color::rgb(200, 200, 200))
                                .border_radius(Pixels(3.0))
                                .height(Pixels(28.0))
                                .width(Stretch(1.0));

                                Element::new(cx)
                                    .height(Pixels(1.0))
                                    .background_color(Color::rgb(50, 50, 50))
                                    .top(Pixels(6.0))
                                    .bottom(Pixels(6.0));

                                List::new(cx, EditorData::presets, |cx, _, item| {
                                    HStack::new(cx, move |cx| {
                                        Button::new(
                                            cx,
                                            move |cx| {
                                                let current_name = item.get(cx).name.clone();
                                                cx.emit(EditorEvent::LoadPreset(current_name));
                                            },
                                            move |cx| {
                                                Label::new(cx, item.map(|p| p.name.clone()))
                                                    .width(Stretch(1.0))
                                            },
                                        )
                                        .background_color(Color::rgb(40, 45, 60))
                                        .color(Color::rgb(200, 200, 200))
                                        .border_radius(Pixels(3.0))
                                        .width(Stretch(1.0));

                                        Button::new(
                                            cx,
                                            move |cx| {
                                                let current_name = item.get(cx).name.clone();
                                                cx.emit(EditorEvent::InitiateDelete(current_name));
                                            },
                                            |cx| Label::new(cx, "X"),
                                        )
                                        .background_color(Color::rgb(230, 90, 90))
                                        .color(Color::rgb(200, 200, 200))
                                        .border_radius(Pixels(3.0))
                                        .left(Pixels(4.0))
                                        .width(Pixels(30.0));
                                    })
                                    .height(Pixels(30.0))
                                    .child_top(Stretch(1.0))
                                    .child_bottom(Stretch(1.0));
                                });
                            })
                            .background_color(Color::rgb(20, 20, 30))
                            .position_type(PositionType::SelfDirected)
                            .top(Pixels(30.0))
                            .right(Pixels(0.0))
                            .width(Pixels(200.0));
                        }
                    });

                    Binding::new(cx, EditorData::pending_delete, |cx, pending| {
                        if let Some(name) = pending.get(cx) {
                            ZStack::new(cx, |cx| {
                                Element::new(cx).background_color(Color::rgba(0, 0, 0, 150));

                                VStack::new(cx, |cx| {
                                    Label::new(cx, &format!("Delete '{}'?", name))
                                        .color(Color::rgb(200, 200, 200))
                                        .top(Pixels(10.0))
                                        .bottom(Pixels(15.0));

                                    HStack::new(cx, |cx| {
                                        Button::new(
                                            cx,
                                            |cx| cx.emit(EditorEvent::ConfirmDelete),
                                            |cx| Label::new(cx, "Yes"),
                                        )
                                        .background_color(Color::rgb(230, 90, 90))
                                        .color(Color::rgb(200, 200, 200))
                                        .border_radius(Pixels(3.0))
                                        .width(Pixels(60.0));

                                        Button::new(
                                            cx,
                                            |cx| cx.emit(EditorEvent::CancelDelete),
                                            |cx| Label::new(cx, "No"),
                                        )
                                        .background_color(Color::rgb(40, 45, 60))
                                        .color(Color::rgb(200, 200, 200))
                                        .border_radius(Pixels(3.0))
                                        .left(Pixels(10.0))
                                        .width(Pixels(60.0));
                                    })
                                    .child_space(Stretch(1.0));
                                })
                                .background_color(Color::rgb(30, 30, 40))
                                .border_radius(Pixels(5.0))
                                .width(Pixels(220.0))
                                .height(Pixels(100.0))
                                .child_space(Stretch(1.0));
                            });
                        }
                    });

                    Binding::new(cx, EditorData::pending_save, |cx, pending| {
                        if pending.get(cx) {
                            ZStack::new(cx, |cx| {
                                Element::new(cx).background_color(Color::rgba(0, 0, 0, 150));

                                VStack::new(cx, |cx| {
                                    Label::new(cx, "Save New Preset")
                                        .color(Color::rgb(200, 200, 200))
                                        .top(Pixels(5.0))
                                        .bottom(Pixels(15.0));

                                    Textbox::new(cx, EditorData::save_name)
                                        .on_edit(|cx, text| {
                                            cx.emit(EditorEvent::SetPresetNameInput(text))
                                        })
                                        .background_color(Color::rgb(13, 10, 22))
                                        .color(Color::rgb(210, 205, 235))
                                        .border_width(Pixels(1.0))
                                        .border_color(Color::rgb(50, 50, 60))
                                        .border_radius(Pixels(3.0))
                                        .height(Pixels(30.0))
                                        .width(Stretch(1.0))
                                        .bottom(Pixels(15.0));

                                    HStack::new(cx, |cx| {
                                        Button::new(
                                            cx,
                                            |cx| cx.emit(EditorEvent::ConfirmSave),
                                            |cx| Label::new(cx, "Save"),
                                        )
                                        .background_color(Color::rgb(70, 35, 120))
                                        .color(Color::rgb(220, 200, 255))
                                        .border_radius(Pixels(3.0))
                                        .width(Pixels(65.0));

                                        Button::new(
                                            cx,
                                            |cx| cx.emit(EditorEvent::CancelSave),
                                            |cx| Label::new(cx, "Cancel"),
                                        )
                                        .background_color(Color::rgb(40, 45, 60))
                                        .color(Color::rgb(200, 200, 200))
                                        .border_radius(Pixels(3.0))
                                        .left(Pixels(10.0))
                                        .width(Pixels(65.0));
                                    })
                                    .child_space(Stretch(1.0));
                                })
                                .background_color(Color::rgb(30, 30, 40))
                                .border_radius(Pixels(5.0))
                                .width(Pixels(240.0))
                                .height(Pixels(130.0))
                                .child_space(Stretch(1.0));
                            });
                        }
                    });
                });
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if let Ok(guard) = self.engine.try_lock() {
            if let Some(engine) = guard.as_ref() {
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
        &[Vst3SubCategory::Generator, Vst3SubCategory::Tools];
}

nih_export_clap!(LuaSound);
nih_export_vst3!(LuaSound);
