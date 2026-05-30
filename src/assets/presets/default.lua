function process_block(left, right, ctx, dsp)
    -- ctx: {sample_rate, tempo, is_playing}
    -- dsp: {delay_read(name, samples), delay_write(name, value)}

    for i = 1, #left do
        -- You can manipulate individual samples here
        -- left[i] = left[i] * 0.5 
        -- right[i] = right[i] * 0.5
    end

    -- example: passthrough
    return left, right
end