use crate::{engine::LuaEngine, presets::Preset};
use nih_plug_vizia::vizia::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum EditorEvent {
    SetScript(String),
    Apply,
    Import,
    Export,
    Imported(String),
    Exported,
    TogglePresetMenu,
    InitiateDelete(String),
    ConfirmDelete,
    CancelDelete,
    InitiateSave,
    SetPresetNameInput(String),
    ConfirmSave,
    CancelSave,
    LoadPreset(String),
}

#[derive(Lens, Clone)]
pub struct EditorData {
    pub script: String,
    pub status: String,
    pub status_ok: bool,
    pub is_dirty: bool,
    pub presets: Vec<Preset>,
    pub is_preset_menu_open: bool,
    pub pending_save: bool,
    pub save_name: String,
    pub pending_delete: Option<String>,
    #[lens(ignore)]
    pub engine: Arc<Mutex<Option<LuaEngine>>>,
    #[lens(ignore)]
    pub script_store: Arc<Mutex<String>>,
}

impl Model for EditorData {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|e: &EditorEvent, _| match e {
            EditorEvent::SetScript(s) => {
                self.script = s.clone();
                if let Ok(mut store) = self.script_store.lock() {
                    *store = s.to_string();
                }
                self.is_dirty = true;
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
                        self.is_dirty = false;
                    }
                    Err(e) => {
                        *self.engine.lock().unwrap() = None;
                        self.status = format!("Error: {}", e);
                        self.status_ok = false;
                    }
                }
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
                self.is_dirty = true;
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

            EditorEvent::TogglePresetMenu => {
                self.is_preset_menu_open = !self.is_preset_menu_open;
                if !self.is_preset_menu_open {
                    self.pending_save = false;
                    self.pending_delete = None;
                }
            }

            EditorEvent::InitiateDelete(name) => self.pending_delete = Some(name.clone()),

            EditorEvent::CancelDelete => self.pending_delete = None,

            EditorEvent::ConfirmDelete => {
                if let Some(name) = self.pending_delete.take() {
                    let _ = crate::presets::delete_preset(&name);
                    self.presets = crate::presets::load_presets();
                }
            }

            EditorEvent::InitiateSave => {
                self.pending_save = true;
                self.save_name = "My Preset".to_owned();
            }

            EditorEvent::SetPresetNameInput(name) => self.save_name = name.clone(),

            EditorEvent::ConfirmSave => {
                let _ = crate::presets::save_preset(&self.save_name.clone(), &self.script);
                self.presets = crate::presets::load_presets();
            }

            EditorEvent::CancelSave => self.pending_save = false,

            EditorEvent::LoadPreset(name) => {
                if let Some(preset) = self.presets.iter().find(|p| &p.name == name) {
                    self.script = preset.script.clone();

                    if let Ok(mut store) = self.script_store.lock() {
                        *store = self.script.clone();
                    }

                    self.status = format!("Loaded {}", name);
                    self.status_ok = true;
                    self.is_dirty = true;
                    self.is_preset_menu_open = false;
                }
            }
        });
    }
}
