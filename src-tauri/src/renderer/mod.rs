#[cfg(target_os = "macos")]
pub mod canvas;
#[cfg(target_os = "macos")]
pub mod draw;

#[cfg(test)]
mod tests {
    #[test]
    fn renderer_module_compiles() {
        assert!(true);
    }
}
