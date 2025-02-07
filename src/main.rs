use glium::{implement_vertex, uniform, Surface};

mod teapot;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

fn main() {
    let event_loop = glium::winit::event_loop::EventLoop::builder()
        .build()
        .expect("event loop building");
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .with_title("Voxel Engine V1")
        .build(&event_loop);

    // let shape = vec![
    //     Vertex { position: [-0.5, -0.5], color: [1.0, 0.0, 0.0] },
    //     Vertex { position: [ 0.0,  0.5], color: [0.0, 1.0, 0.0] },
    //     Vertex { position: [ 0.5, -0.25], color: [0.0, 0.0, 1.0] }
    // ];

    // let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();

    // let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                                                            &teapot::INDICES).unwrap();

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;
        out vec3 v_position;

        uniform mat4 view;
        uniform mat4 perspective;
        uniform mat4 model;

        void main() {
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * model * vec4(position, 1.0);
            v_position = gl_Position.xyz / gl_Position.w;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        in vec3 v_position;

        out vec4 color;
        uniform vec3 u_light;

        const vec3 ambient_color = vec3(0.2, 0.0, 0.0);
        const vec3 diffuse_color = vec3(0.6, 0.0, 0.0);
        const vec3 specular_color = vec3(1.0, 1.0, 1.0);

        void main() {
            float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);

            vec3 camera_dir = normalize(-v_position);
            vec3 half_direction = normalize(normalize(u_light) + camera_dir);
            float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

            color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);

        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    #[allow(deprecated)]
    let _ = event_loop.run(move |event, window_target| {
        match event {
            glium::winit::event::Event::WindowEvent { event, .. } => match event {
                glium::winit::event::WindowEvent::CloseRequested => window_target.exit(),
                glium::winit::event::WindowEvent::Resized(window_size) => {
                    display.resize(window_size.into());
                },
                glium::winit::event::WindowEvent::RedrawRequested => {
                    let mut frame = display.draw();
                    // the direction of the light
                    let light = [-1.0, -0.4, -0.9f32];

                    let perspective = {
                        let (width, height) = frame.get_dimensions();
                        let aspect_ratio = height as f32 / width as f32;
                    
                        let fov: f32 = 3.141592 / 3.0;
                        let zfar = 1024.0;
                        let znear = 0.1;
                    
                        let f = 1.0 / (fov / 2.0).tan();
                    
                        [
                            [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
                            [         0.0         ,     f ,              0.0              ,   0.0],
                            [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
                            [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
                        ]
                    };

                    let view = view_matrix(&[2.0, -1.0, 1.0], &[-2.0, 1.0, 1.0], &[0.0, 1.0, 0.0]);

                    let uniforms = uniform! {
                        model: [
                            [0.1, 0.0, 0.0, 0.0],
                            [0.0, 0.1, 0.0, 0.0],
                            [0.0, 0.0, 0.1, 0.0],
                            [0.0, 0.0, 2.0, 1.0f32]
                        ],
                        u_light: light,
                        perspective: perspective,
                        view: view,
                    };

                    let params = glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::draw_parameters::DepthTest::IfLess,
                            write: true,
                            .. Default::default()
                        },
                        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullingDisabled,
                        .. Default::default()
                    };

                    frame.clear_color_and_depth((0.4, 1.0, 1.0, 1.0), 1.0);
                    frame.draw((&positions, &normals), &indices, &program, &uniforms,
                                &params).unwrap();
                    frame.finish().unwrap();
                }
                _ => (),
            },
            glium::winit::event::Event::AboutToWait => {
                window.request_redraw();
            },
            _ => (),
        };
    });
}


fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}