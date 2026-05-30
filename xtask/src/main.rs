use cargo_toml::Manifest;
use std::fs;
use std::path::Path;

fn main() -> nih_plug_xtask::Result<()> {
    if Path::new("target/bundled").exists() {
        fs::remove_dir_all("target/bundled").expect("Failed to clean target/bundled");
    }
    
    nih_plug_xtask::main()?;

    let manifest = Manifest::from_path("Cargo.toml").expect("Could not find Cargo.toml");
    let package = manifest.package();
    let version = package.version.clone().unwrap(); 

    let base = "target/bundled";

    let old_vst_bundle = format!("{}/lua-sound.vst3", base);
    let new_vst_bundle = format!("{}/LuaSound-v{}.vst3", base, version);
    if Path::new(&old_vst_bundle).exists() {
        fs::rename(&old_vst_bundle, &new_vst_bundle)?;
        
        let old_bin = format!("{}/Contents/x86_64-win/lua-sound.vst3", new_vst_bundle);
        let new_bin = format!("{}/Contents/x86_64-win/LuaSound-v{}.vst3", new_vst_bundle, version);
        if Path::new(&old_bin).exists() {
            fs::rename(old_bin, new_bin)?;
        }
    }

    let old_clap = format!("{}/lua-sound.clap", base);
    let new_clap = format!("{}/LuaSound-v{}.clap", base, version);
    if Path::new(&old_clap).exists() {
        fs::rename(&old_clap, &new_clap)?;
    }
    
    println!("Successfully renamed all artifacts to \"LuaSound-v{}\"", version);
    Ok(())
}