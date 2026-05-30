local bit_depth       = 8  -- bit depth (1-16, lower = crunchier)
local sample_rate_div = 4  -- sample rate divisor (1 = no decimation, higher = more lo-fi)

local held_l = 0.0
local held_r = 0.0
local hold_counter = 0

function process_block(left, right, ctx, dsp)
    local levels = 2 ^ bit_depth

    local out_left  = {}
    local out_right = {}

    for i = 1, #left do
        hold_counter = hold_counter + 1

        if hold_counter >= sample_rate_div then
            hold_counter = 0
            held_l = math.floor(left[i]  * levels + 0.5) / levels
            held_r = math.floor(right[i] * levels + 0.5) / levels
        end

        out_left[i]  = held_l
        out_right[i] = held_r
    end

    return out_left, out_right
end
