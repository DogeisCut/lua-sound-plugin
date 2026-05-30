local drive = 4.0   -- input gain before clipping (higher = more distortion)
local level = 0.5   -- output level

function process_block(left, right, ctx, dsp)
    for i = 1, #left do
        left[i]  = math.tanh(left[i]  * drive) * level
        right[i] = math.tanh(right[i] * drive) * level
    end

    return left, right
end
