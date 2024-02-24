pub const VERTEX_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec3 v_pos;
    layout (location = 1) in vec2 v_uv;
    layout (location = 2) in vec3 v_norm;

    layout (location = 0) out vec3 a_color;

    void main() {
        a_color = vec3(1.0, 1.0, 1.0);
        gl_Position = vec4(v_pos, 1.0);
    }
"#;

pub const FRAGMENT_SRC: &str = r#"
    #version 450 core
    layout (location = 0) in vec3 a_color;
    out vec4 f_color;

    void main() {
       f_color = vec4(a_color, 1.0);
    }
"#;

pub const SPRITE_VERTEX_SRC: &str = r#"
    #version 450 core
    layout (location = 0) out vec2 a_uv;
    layout (location = 1) out vec2 a_uv_min;
    layout (location = 2) out vec2 a_uv_extent;
    layout (location = 3) out vec2 a_uv_scroll;


    layout (binding = 0) uniform Sprite {
        mat4 transform;
        vec2 uv_min;
        vec2 uv_extent;
        vec2 uv_scroll;

    };
    layout (binding = 1) uniform Camera {
        mat4 camera;
    };


    void main() {
        int i = gl_VertexID + 1;
        float x = float(1 - (i & 2));
        float y = float(1 - ((i & 1) * 2));
        a_uv = vec2((x+1) / 2,1-(y+1)/2) * uv_extent;
        a_uv_min = uv_min;
        a_uv_extent = uv_extent;
        a_uv_scroll = uv_scroll;

        gl_Position = camera * transform * vec4(x, y, 0, 1.0);
    }
"#;

pub const SPRITE_FRAGMENT_SRC: &str = r#"
    #version 450 core
    out vec4 f_color;
    layout (location = 0) in vec2 a_uv;
    layout (location = 1) in vec2 a_uv_min;
    layout (location = 2) in vec2 a_uv_extent;
    layout (location = 3) in vec2 a_uv_scroll;

    layout (binding = 3) uniform sampler2D u_texture;

    void main() {
        vec2 uv = mod((a_uv + a_uv_scroll), a_uv_extent) + a_uv_min;
        f_color = texture(u_texture, uv);
    }
"#;
