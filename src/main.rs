use glfw::{Action, Context, Key};
use gl::types::*;

use std::sync::mpsc::Receiver;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::os::raw::c_void;

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 800;

// GLSL for vertex shader
const vertexShaderSource: &str = r#"
    #version 330 core
    layout (location = 0) in vec3 aPos;
    void main() {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
"#;

// GLSL for fragment shader
const fragmentShaderSource: &str = r#"
    #version 330 core
    out vec4 FragColor;
    void main() {
        FragColor = vec4(1.0f, 0.5f, 0.2f, 1.0f);
    }
"#;

fn main() {

    // Initialize and configure glfw
    // A token from which to call various GLFW functions. It can be obtained by calling the init
    // function. This cannot be sent to other tasks, and should only be initialized on the main
    // platform thread. Whilst this might make performing some operations harder, this is to ensure
    // thread safety is enforced statically. The context can be safely cloned or implicitly copied
    // if need be for convenience.
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // https://www.glfw.org/docs/latest/window.html#window_hints
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    // https://www.glfw.org/docs/latest/window.html#GLFW_OPENGL_FORWARD_COMPAT_hint
    // See the note above the anchor
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    // Let’s make a window
    // create_window: width, height, title, mode
    // returns a Option<(Window, Receiver<(f64, WindowEvent)>)>
    // https://docs.rs/glfw/0.29.0/glfw/struct.Glfw.html#method.create_window
    let (mut window, events) = glfw.create_window(SCR_WIDTH, SCR_HEIGHT, "Hello, World!", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // Make this window the current context on the current thread
    window.make_current();

    // TODO: look up what these do.
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // Load all OpenGL function pointers
    //
    // Load each OpenGL symbol using a custom load function. This allows for the use of functions
    // like glfwGetProcAddress or SDL_GL_GetProcAddress.
    //
    // Returns the address of the specified client API or extension function if it is supported by
    // the context associated with this Window. If this Window is not the current context, it will
    // make it the current context.
    // https://docs.rs/glfw/0.29.0/glfw/struct.Window.html#method.get_proc_address
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    // Our shader program
    let (shaderProgram, VAO) = unsafe {

        // vertex shader
        let vertexShader = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(vertexShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(vertexShader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertexShader);
        // check for shader compile errors
        let mut success = gl::FALSE as GLint;
        let mut infoLog = Vec::with_capacity(512);
        infoLog.set_len(512 - 1); // skips trailig null character
        gl::GetShaderiv(vertexShader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(vertexShader, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::VERTEX::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
        }

        // fragment shader
        let fragmentShader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(fragmentShaderSource.as_bytes()).unwrap();
        gl::ShaderSource(fragmentShader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragmentShader);
        // check for shader compile errors
        gl::GetShaderiv(fragmentShader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(fragmentShader, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::FRAGMENT::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
        }

        // link shaders
        let shaderProgram = gl::CreateProgram();
        gl::AttachShader(shaderProgram, vertexShader);
        gl::AttachShader(shaderProgram, fragmentShader);
        gl::LinkProgram(shaderProgram);
        // check for linking errors
        gl::GetProgramiv(shaderProgram, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(shaderProgram, 512, ptr::null_mut(), infoLog.as_mut_ptr() as *mut GLchar);
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", str::from_utf8(&infoLog).unwrap());
        }
        gl::DeleteShader(vertexShader);
        gl::DeleteShader(fragmentShader);

        // set up vertex data
        let vertices: [f32; 12] = [
            // x     y    z
             0.5,  0.5, 0.0, // left
             0.5, -0.5, 0.0, // right
            -0.5, -0.5, 0.0, // left
            -0.5,  0.5, 0.0, // top
             //      t
             //
             //  l       r
        ];
        let indices = [
            0, 1, 3,
            1, 2, 3,
        ];
        let (mut VBO, mut VAO, mut EBO) = (0, 0, 0);
        // “vertex array object”
        gl::GenVertexArrays(1, &mut VAO);
        // “vertex buffer object“
        gl::GenBuffers(1, &mut VBO);
        gl::GenBuffers(1, &mut EBO);
        gl::BindVertexArray(VAO);

        gl::BindBuffer(gl::ARRAY_BUFFER, VBO);

        // target, size, data, usage
        gl::BufferData(gl::ARRAY_BUFFER,
                       (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &vertices[0] as *const f32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, EBO);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                       (indices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                       &indices[0] as *const i32 as *const c_void,
                       gl::STATIC_DRAW);

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);

        // note that this is allowed, the call to gl::VertexAttribPointer registered VBO as the vertex attribute's bound vertex buffer object so afterwards we can safely unbind
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        // You can unbind the VAO afterwards so other VAO calls won't accidentally modify this VAO, but this rarely happens. Modifying other
        // VAOs requires a call to glBindVertexArray anyways so we generally don't unbind VAOs (nor VBOs) when it's not directly necessary.
        gl::BindVertexArray(0);

        (shaderProgram, VAO)
    };

    // Render loop
    while !window.should_close() {
        // events handler
        process_events(&mut window, &events);

        // Let’s set the colour the window
        unsafe {
            gl::ClearColor(0.4, 0.2, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw our first triangle
            gl::UseProgram(shaderProgram);

            gl::BindVertexArray(VAO);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        // Notices how these next two function calls relate to what the process_events functio is
        // doing.
        // “This gives us an easy way to check for specific key presses and react accordingly every frame.”

        // The glfwSwapBuffers will swap the color buffer (a large buffer that contains color
        // values for each pixel in GLFW's window) that has been used to draw in during this
        // iteration and show it as output to the screen.
        window.swap_buffers();

        // The glfwPollEvents function checks if any events are triggered (like keyboard input or
        // mouse movement events), updates the window state, and calls the corresponding functions
        // (which we can set via callback methods)
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            // make sure the viewport matches the new window dimensions; note that width and
            // height will be significantly larger than specified on retina displays.
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe { gl::Viewport(0, 0, width, height) }
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            glfw::WindowEvent::Key(Key::F, _, Action::Press, _) => window.maximize(),
            _ => {}
        }
    }
}
