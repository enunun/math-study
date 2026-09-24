//! 正多面体の頂点と面．種類ごとに，原点を中心とする基準の頂点と，外から見て反時計回りに頂点を並べた面を
//! 持ち，中心と半径(中心から頂点までの距離)に合わせて置いた複体(`Complex`)にする．
//!
//! 面の並びは，頂点の座標の凸包から求めたもので，正しさ(平らで外向き，稜の長さが等しい，オイラーの
//! 公式など)は`tests/polyhedron.rs`が確かめる．

use crate::scene::{Complex, Polyhedron, Solid};

/// 黄金比．正12面体・正20面体の頂点の座標に使う．
const PHI: f64 = 1.618_033_988_749_895;
/// 黄金比の逆数．
const INV_PHI: f64 = PHI - 1.0;

/// 正4面体の頂点．立方体の頂点を1つおきに取る．
const TETRAHEDRON_VERTICES: [[f64; 3]; 4] = [
    [1.0, 1.0, 1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
];
const TETRAHEDRON_FACES: [[usize; 3]; 4] = [[0, 1, 2], [1, 0, 3], [0, 2, 3], [1, 3, 2]];

/// 立方体(正6面体)の頂点．
const CUBE_VERTICES: [[f64; 3]; 8] = [
    [1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
];
const CUBE_FACES: [[usize; 4]; 6] = [
    [1, 0, 2, 3],
    [0, 1, 5, 4],
    [2, 0, 4, 6],
    [1, 3, 7, 5],
    [3, 2, 6, 7],
    [4, 5, 7, 6],
];

/// 正8面体の頂点．座標軸上の6点．
const OCTAHEDRON_VERTICES: [[f64; 3]; 6] = [
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];
const OCTAHEDRON_FACES: [[usize; 3]; 8] = [
    [0, 2, 4],
    [0, 5, 2],
    [0, 4, 3],
    [0, 3, 5],
    [4, 2, 1],
    [2, 5, 1],
    [3, 4, 1],
    [5, 3, 1],
];

/// 正12面体の頂点．立方体の8頂点と，3方向の黄金長方形の頂点を合わせたもの．
const DODECAHEDRON_VERTICES: [[f64; 3]; 20] = [
    [1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [0.0, INV_PHI, PHI],
    [0.0, INV_PHI, -PHI],
    [0.0, -INV_PHI, PHI],
    [0.0, -INV_PHI, -PHI],
    [INV_PHI, PHI, 0.0],
    [INV_PHI, -PHI, 0.0],
    [-INV_PHI, PHI, 0.0],
    [-INV_PHI, -PHI, 0.0],
    [PHI, 0.0, INV_PHI],
    [PHI, 0.0, -INV_PHI],
    [-PHI, 0.0, INV_PHI],
    [-PHI, 0.0, -INV_PHI],
];
const DODECAHEDRON_FACES: [[usize; 5]; 12] = [
    [16, 17, 1, 12, 0],
    [2, 16, 0, 8, 10],
    [0, 12, 14, 4, 8],
    [1, 17, 3, 11, 9],
    [12, 1, 9, 5, 14],
    [17, 16, 2, 13, 3],
    [13, 2, 10, 6, 15],
    [3, 13, 15, 7, 11],
    [4, 14, 5, 19, 18],
    [10, 8, 4, 18, 6],
    [9, 11, 7, 19, 5],
    [7, 15, 6, 18, 19],
];

/// 正20面体の頂点．3枚の黄金長方形(1：φ)の頂点を合わせたもの．
const ICOSAHEDRON_VERTICES: [[f64; 3]; 12] = [
    [0.0, 1.0, PHI],
    [0.0, 1.0, -PHI],
    [0.0, -1.0, PHI],
    [0.0, -1.0, -PHI],
    [1.0, PHI, 0.0],
    [1.0, -PHI, 0.0],
    [-1.0, PHI, 0.0],
    [-1.0, -PHI, 0.0],
    [PHI, 0.0, 1.0],
    [PHI, 0.0, -1.0],
    [-PHI, 0.0, 1.0],
    [-PHI, 0.0, -1.0],
];
const ICOSAHEDRON_FACES: [[usize; 3]; 20] = [
    [8, 0, 2],
    [2, 0, 10],
    [4, 6, 0],
    [8, 4, 0],
    [0, 6, 10],
    [9, 3, 1],
    [1, 3, 11],
    [4, 1, 6],
    [4, 9, 1],
    [1, 11, 6],
    [5, 2, 7],
    [5, 8, 2],
    [2, 10, 7],
    [3, 5, 7],
    [9, 5, 3],
    [3, 7, 11],
    [4, 8, 9],
    [9, 8, 5],
    [6, 11, 10],
    [10, 11, 7],
];

/// 基準の頂点(原点が中心，どの頂点も原点から同じ距離)と面．
fn reference(solid: Solid) -> (&'static [[f64; 3]], Vec<Vec<usize>>) {
    fn faces<const N: usize>(faces: &[[usize; N]]) -> Vec<Vec<usize>> {
        faces.iter().map(|face| face.to_vec()).collect()
    }
    match solid {
        Solid::Tetrahedron => (&TETRAHEDRON_VERTICES, faces(&TETRAHEDRON_FACES)),
        Solid::Cube => (&CUBE_VERTICES, faces(&CUBE_FACES)),
        Solid::Octahedron => (&OCTAHEDRON_VERTICES, faces(&OCTAHEDRON_FACES)),
        Solid::Dodecahedron => (&DODECAHEDRON_VERTICES, faces(&DODECAHEDRON_FACES)),
        Solid::Icosahedron => (&ICOSAHEDRON_VERTICES, faces(&ICOSAHEDRON_FACES)),
    }
}

impl Polyhedron {
    /// 中心と半径に合わせて置いた，同じ形の複体．識別子とスタイルは，そのまま受け継ぐ．
    #[must_use]
    pub fn to_complex(&self) -> Complex {
        let (vertices, faces) = reference(self.solid);
        let vertices = vertices
            .iter()
            .map(|vertex| {
                let length = vertex.iter().map(|c| c * c).sum::<f64>().sqrt();
                let scale = self.radius / length;
                [
                    self.center[0] + vertex[0] * scale,
                    self.center[1] + vertex[1] * scale,
                    self.center[2] + vertex[2] * scale,
                ]
            })
            .collect();
        Complex {
            id: self.id.clone(),
            vertices,
            faces,
            style: self.style,
            transform: self.transform.clone(),
        }
    }
}
