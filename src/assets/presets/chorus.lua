local rate_hz       = 0.8   -- LFO speed in Hz
local depth_ms      = 10.0  -- modulation depth in milliseconds
local base_delay_ms = 20.0  -- base delay time in milliseconds
local mix           = 0.5   -- wet/dry mix (0.0 = dry, 1.0 = fully wet)

local phase_l = 0.0
local phase_r = math.pi  -- offset right channel for a wider stereo image

function process_block(left, right, ctx, dsp)
    local phase_inc    = (2.0 * math.pi * rate_hz) / ctx.sample_rate
    local depth_samp   = (depth_ms    / 1000.0) * ctx.sample_rate
    local base_samp    = math.floor((base_delay_ms / 1000.0) * ctx.sample_rate)
    local two_pi       = 2.0 * math.pi

    local out_left  = {}
    local out_right = {}

    for i = 1, #left do
        local delay_l = base_samp + math.floor(depth_samp * math.sin(phase_l))
        local delay_r = base_samp + math.floor(depth_samp * math.sin(phase_r))

        local wet_l = dsp:delay_read("chorus_l", delay_l)
        local wet_r = dsp:delay_read("chorus_r", delay_r)

        out_left[i]  = left[i]  * (1.0 - mix) + wet_l * mix
        out_right[i] = right[i] * (1.0 - mix) + wet_r * mix

        dsp:delay_write("chorus_l", left[i])
        dsp:delay_write("chorus_r", right[i])

        phase_l = phase_l + phase_inc
        phase_r = phase_r + phase_inc
        if phase_l > two_pi then phase_l = phase_l - two_pi end
        if phase_r > two_pi then phase_r = phase_r - two_pi end
    end

    return out_left, out_right
end
