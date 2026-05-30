local gain_db = -6.0  -- gain in decibels (negative = quieter, positive = louder)

function process_block(left, right, ctx, dsp)
    local gain_linear = 10 ^ (gain_db / 20.0)

    for i = 1, #left do
        left[i] = left[i] * gain_linear
        right[i] = right[i] * gain_linear
    end

    return left, right
end
