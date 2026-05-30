# Lua Sound

**Lua Sound** is an audio plugin that allows you to write real-time DSP (Digital Signal Processing) effects using the Lua scripting language. Designed for rapid prototyping and custom sound design, Lua Sound integrates directly into your DAW as a VST3 or CLAP plugin.

The core logic and implementation are contained within lib.rs.

## Features

* **Two Processing Modes:**
  * **Simple Mode:** Process audio sample-by-sample. Ideal for simple distortion, gain, or waveshaping.
  * **Advanced Mode:** Process entire audio blocks. Ideal for delay lines, filters, or complex temporal effects.
* **Real-time Scripting:** Update and "Run" your code without recompiling the plugin.
* **Shared State:** Access your DAW's transport information (BPM, play state, sample rate) directly within Lua.
* **Delay Buffer API:** Built-in support for read/write delay buffers.
* **Import/Export:** Save your creative scripts as .lua files.

## Lua API

### Simple Mode
Define a process function that runs on every sample:

```lua
function process(left, right)
    -- Apply your DSP here

    return left, right
end
```

### Advanced Mode
Define a process_block function for block-based processing:

```lua
function process_block(left, right, ctx, dsp)
    -- ctx: {sample_rate, tempo, is_playing}
    -- dsp: {delay_read(name, samples), delay_write(name, value)}
    
    return left, right
end
```

## Building
This project is built using Rust (https://www.rust-lang.org/) and the nih-plug (https://nih-plug.rs/) framework.

1. Install Rust.
2. Clone the repository.
3. Build the plugin using cargo:

```bash
cargo build --release
```

4. Rename the resulting `lua_sound_plugin.dll` file to `LuaSound.vst3`
5. Place the file in `C:\Program Files\Common Files\VST3`

This process will change in the future.

## License
This project is licensed under the **GNU General Public License v3.0 (GPLv3)**. See the `LICENSE.txt` file for more details.