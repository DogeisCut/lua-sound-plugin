local rate_hz = 4.0  -- LFO speed in Hz
local depth   = 0.8  -- modulation depth (0.0 = no effect, 1.0 = full on/off)

local phase = 0.0

function process_block(left, right, ctx, dsp)
    local phase_inc = (2.0 * math.pi * rate_hz) / ctx.sample_rate
    local two_pi    = 2.0 * math.pi

    for i = 1, #left do
        -- LFO oscillates between (1 - depth) and 1.0
        local lfo = (1.0 - depth) + depth * (0.5 + 0.5 * math.sin(phase))

        left[i]  = left[i]  * lfo
        right[i] = right[i] * lfo

        phase = phase + phase_inc
        if phase > two_pi then phase = phase - two_pi end
    end

    return left, right
end
