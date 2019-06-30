use glfw::Context;

fn main() {
    // initializes the GLFW library
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // create window
    // The GLFWwindow object encapsulates both a window and a context.
    // As the window and context are inseparably linked, the window object also serves as the context handle.
    // https://www.glfw.org/docs/latest/window_guide.html#window_object
    let (mut window, _events) = glfw.create_window(800, 600, "Hello, World!", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    // make this window the current context on the current thread
    window.make_current();

    //// render loop
    while !window.should_close() {
        glfw.poll_events();
    }
}
