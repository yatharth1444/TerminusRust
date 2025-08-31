# 📟TerminusRust

A minimal Rust-based terminal emulator prototype combining PTY shell integration with OpenGL rendering.

## What I Built

This project creates an OpenGL-enabled window using `glutin` and `winit`, then spawns a Unix shell inside a pseudo-terminal (PTY) using the `nix` crate. It asynchronously reads the shell output and captures keyboard input, forwarding it to the shell process. The current version prints shell output to the console as a placeholder for future graphical rendering. The architecture allows incremental addition of terminal features like font rendering, ANSI escape code parsing, configuration, and GPU-accelerated text display.

## Future Prospects

- Implement GPU accelerated font rendering with OpenGL for crisp, high-performance terminal output.
- Add full ANSI escape code parsing for colored and styled terminal output.
- Support advanced terminal features like mouse input, window resizing, and tabs.
- Make the emulator cross-platform by refining system integration (Windows/macOS support).
- Optimize performance and resource usage for a production-quality terminal emulator.

## What I Learned

- How to integrate Rust with low-level OS features like PTYs for shell management.
- Basics of asynchronous IO and concurrency in Rust using threads and mutexes.
- Setting up OpenGL contexts and windows using Rust crates like `glutin` and `winit`.
- Managing system resources and unsafe code patterns safely with Rust's ownership model.
- Building an event-driven architecture that ties system input, shell processes, and graphics together.

---

This project lays the foundation for building a fully-featured, GPU-accelerated terminal emulator completely in Rust, showcasing skills in systems programming, graphics, and Rust concurrency.

