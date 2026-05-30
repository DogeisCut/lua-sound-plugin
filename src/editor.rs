use crate::engine::LuaEngine;
use crate::{ADVANCED_SCRIPT, DEFAULT_SCRIPT};
use nih_plug_vizia::vizia::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum EditorEvent {
    SetScript(String),
    Apply,
    Import,
    Export,
    Imported(String),
    Exported,
    ToggleMode,
}

#[derive(Lens, Clone)]
pub struct EditorData {
    pub script: String,
    pub status: String,
    pub status_ok: bool,
    pub is_advanced: bool,
    #[lens(ignore)]
    pub engine: Arc<Mutex<Option<LuaEngine>>>,
    #[lens(ignore)]
    pub script_store: Arc<Mutex<String>>,
    #[lens(ignore)]
    pub is_advanced_store: Arc<AtomicBool>,
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
