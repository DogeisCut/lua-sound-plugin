use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Preset {
    pub name: String,
    pub script: String,
}

pub const BUNDLED_PRESETS: &[(&str, &str)] = &[
    ("Default", include_str!("assets/presets/default.lua")),
    ("Gain", include_str!("assets/presets/gain.lua")),
    ("Delay", include_str!("assets/presets/delay.lua")),
    ("Bitcrusher", include_str!("assets/presets/bitcrusher.lua")),
    ("Distortion", include_str!("assets/presets/distortion.lua")),
    ("Tremolo", include_str!("assets/presets/tremolo.lua")),
];

fn bundled() -> Vec<Preset> {
    BUNDLED_PRESETS
        .iter()
        .map(|(name, script)| Preset {
            name: name.to_string(),
            script: script.to_string(),
        })
        .collect()
}

pub fn preset_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("LuaSound").join("presets"))
}

pub fn load_presets() -> Vec<Preset> {
    let Some(dir) = preset_dir() else {
        return bundled();
    };

    if !dir.exists() {
        let _ = reset_to_defaults();
        return bundled();
    }

    let mut presets: Vec<Preset> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "lua" {
                return None;
            }
            let name = path.file_stem()?.to_str()?.to_string();
            let script = std::fs::read_to_string(&path).ok()?;
            Some(Preset { name, script })
        })
        .collect();

    presets.sort_by(|a, b| a.name.cmp(&b.name));
    presets
}

pub fn reset_to_defaults() -> std::io::Result<()> {
    let dir = preset_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No data dir found"))?;

    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;

    for (name, script) in BUNDLED_PRESETS {
        let filename = format!("{}.lua", name.to_lowercase().replace(' ', "_"));
        std::fs::write(dir.join(filename), script)?;
    }

    Ok(())
}

pub fn save_preset(name: &str, script: &str) -> std::io::Result<()> {
    let dir = preset_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No data dir found"))?;
    std::fs::create_dir_all(&dir)?;
    let filename = format!("{}.lua", name.to_lowercase().replace(' ', "_"));
    std::fs::write(dir.join(filename), script)
}
