pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
}

pub fn cube_mesh() -> Mesh {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];

    let indices = vec![
        0, 2, 1, 0, 3, 2,
        4, 5, 6, 4, 6, 7,
        0, 1, 5, 0, 5, 4,
        2, 3, 7, 2, 7, 6,
        0, 4, 7, 0, 7, 3,
        1, 2, 6, 1, 6, 5,
    ];

    Mesh { vertices, indices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_eight_vertices() {
        let mesh = cube_mesh();
        assert_eq!(mesh.vertices.len(), 8);
    }

    #[test]
    fn cube_has_thirty_six_indices() {
        let mesh = cube_mesh();
        assert_eq!(mesh.indices.len(), 36);
    }
}
