use mlua::prelude::*;
use mlua::UserData;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AudioContext {
    pub sample_rate: f32,
    pub tempo: f32,
    pub is_playing: bool,
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
pub struct DspAPI {
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

pub struct LuaEngine {
    lua: Lua,
    dsp: DspAPI,
}

impl LuaEngine {
    pub fn new() -> LuaResult<Self> {
        Ok(Self {
            lua: Lua::new(),
            dsp: DspAPI::new(),
        })
    }

    pub fn load_script(&self, code: &str) -> LuaResult<()> {
        self.lua.load(code).exec()
    }

    pub fn process_sample(&self, left: f32, right: f32) -> LuaResult<(f32, f32)> {
        let process: LuaFunction = self.lua.globals().get("process")?;
        let (l, r): (f32, f32) = process.call((left, right))?;
        Ok((l, r))
    }

    pub fn process_block(
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
