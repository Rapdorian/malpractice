#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: mint::Point3<f32>,
    pub rotation: mint::Quaternion<f32>,
    pub scale: mint::Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: mint::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: mint::Quaternion {
                v: mint::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                s: 1.0,
            },
            scale: mint::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        }
    }
}

impl Transform {
    pub fn with_pos(self, x: f32, y: f32, z: f32) -> Self {
        Self {
            position: mint::Point3 { x, y, z },
            ..self
        }
    }

    pub fn with_scale(self, x: f32, y: f32, z: f32) -> Self {
        Self {
            scale: mint::Vector3 { x, y, z },
            ..self
        }
    }
}

impl Into<mint::ColumnMatrix4<f32>> for Transform {
    fn into(self) -> mint::ColumnMatrix4<f32> {
        let mat: glam::Mat4 = self.into();
        mat.into()
    }
}

impl Into<glam::Mat4> for Transform {
    fn into(self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale.into(),
            self.rotation.into(),
            self.position.into(),
        )
    }
}
