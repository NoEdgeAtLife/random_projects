struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) velocity: vec2<f32>,
    @location(2) particle_type: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert from pixel coordinates to clip space
    let clip_pos = vec2<f32>(
        (input.position.x / 800.0) * 2.0 - 1.0,
        -((input.position.y / 600.0) * 2.0 - 1.0)
    );
    
    output.position = vec4<f32>(clip_pos, 0.0, 1.0);
    
    // Assign different colors based on particle type using switch
    switch input.particle_type {
        case 0u: {
            output.color = vec3<f32>(1.0, 0.1, 0.1); // Red
        }
        case 1u: {
            output.color = vec3<f32>(0.1, 1.0, 0.1); // Green
        }
        case 2u: {
            output.color = vec3<f32>(0.1, 0.1, 1.0); // Blue
        }
        case 3u: {
            output.color = vec3<f32>(1.0, 1.0, 0.1); // Yellow
        }
        default: {
            output.color = vec3<f32>(1.0, 0.1, 1.0); // Purple
        }
    }
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}