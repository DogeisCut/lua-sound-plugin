local delay_time_ms = 500.0 
local feedback      = 0.5

function process_block(left, right, ctx, dsp)     
    local delay_samples = math.floor((delay_time_ms / 1000.0) * ctx.sample_rate)
    
    local out_left = {}
    local out_right = {}
    
    for i = 1, #left do
        
        local delayed_l = dsp:delay_read("delay_l", delay_samples)
        local delayed_r = dsp:delay_read("delay_r", delay_samples)
        
        out_left[i] = left[i] + (delayed_l * feedback)
        out_right[i] = right[i] + (delayed_r * feedback)
        
        
        dsp:delay_write("delay_l", out_left[i])
        dsp:delay_write("delay_r", out_right[i])
    end
    
    return out_left, out_right
end