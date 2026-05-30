function process_block(left, right, ctx, dsp)
    for i = 1, #left do
        -- You can manipulate individual samples here
        -- left[i] = left[i] * 0.5 
        -- right[i] = right[i] * 0.5
    end

    -- example: pass through
    return left, right
end