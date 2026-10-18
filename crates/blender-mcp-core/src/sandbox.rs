use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Execution blocked by safe mode sandbox: dangerous pattern '{pattern}' detected in Python script")]
    ForbiddenPattern { pattern: &'static str },
}

const FORBIDDEN_PATTERNS: &[&str] = &[
    "os.system",
    "os.popen",
    "subprocess.",
    "shutil.rmtree",
    "socket.socket",
    "eval(",
    "exec(",
    "__import__",
    "import ctypes",
    "from ctypes",
    "pty.spawn",
    "os.remove",
    "os.unlink",
];

/// Inspects Python code before sending to live Blender instance.
pub fn validate_blender_code(code: &str) -> Result<(), SandboxError> {
    for &pattern in FORBIDDEN_PATTERNS {
        if code.contains(pattern) {
            return Err(SandboxError::ForbiddenPattern { pattern });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_blender_code() {
        let code = r#"
import bpy
bpy.ops.mesh.primitive_cube_add(size=2.0)
cube = bpy.context.active_object
cube.name = "MyCube"
"#;
        assert!(validate_blender_code(code).is_ok());
    }

    #[test]
    fn test_forbidden_code_blocked() {
        let dangerous = "import os; os.system('rm -rf /')";
        let err = validate_blender_code(dangerous);
        assert!(err.is_err());
    }
}
