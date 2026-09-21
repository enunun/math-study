//! 曲線の標本化．曲がり方に合わせて，点を細かくとる．
//!
//! 定義域を，まず16等分し，各区間で，端と中点，4分の1の点，4分の3の点を見る．
//! 区間の弦から，どれかの点が許容の距離より離れていれば，区間を2つに割って，同じことを繰り返す．
//! 細かい波を，区間の幅より短い周期のせいで取りこぼさないよう，最初の16等分は必ず行う．
//! 値がない点(`None`)と，座標が大きすぎる点は，線を切る点として扱い，切れ目は，二分法で細かく探す．

/// 標本の点．座標の単位は，cmである．
pub type Point = [f64; 2];

/// パラメータの値と，その点．
pub type Sample = (f64, Point);

/// 最初に必ず割る深さ(2の累乗で，16等分)．
const MIN_DEPTH: u32 = 4;
/// 割る深さの上限．
const MAX_DEPTH: u32 = 12;
/// 弦からの許容の距離(cm)．
const TOLERANCE: f64 = 0.003;
/// 座標の絶対値の上限(cm)．これを超える点は，`TikZ`が扱えないので，線を切る点として扱う．
const MAX_COORDINATE: f64 = 300.0;

/// 曲線を標本化する．`f`は，パラメータの値から点を返し，定義域の外や有限でない値では`None`を返す．
/// `None`のところで，線を切る．点が2つに満たない線は捨てる．
pub fn sample(f: impl FnMut(f64) -> Option<Point>, start: f64, end: f64) -> Vec<Vec<Point>> {
    sample_with_parameters(f, start, end)
        .into_iter()
        .map(|path| path.into_iter().map(|(_, point)| point).collect())
        .collect()
}

/// `sample`と同じ点を，パラメータの値つきで返す．空間の曲線で，隠れる部分の切り替わりを探すために使う．
pub fn sample_with_parameters(
    mut f: impl FnMut(f64) -> Option<Point>,
    start: f64,
    end: f64,
) -> Vec<Vec<Sample>> {
    let mut eval = |t: f64| f(t).filter(usable).map(|point| (t, point));
    let ends = [eval(start), eval(end)];
    let mut paths = Paths::default();
    paths.push(ends[0]);
    segment(&mut eval, [start, end], ends, 0, &mut paths);
    paths.finish()
}

fn usable(point: &Point) -> bool {
    point
        .iter()
        .all(|coordinate| coordinate.is_finite() && coordinate.abs() <= MAX_COORDINATE)
}

/// 切れ目で分かれる，点の列の集まり．
#[derive(Default)]
struct Paths {
    finished: Vec<Vec<Sample>>,
    current: Vec<Sample>,
}

impl Paths {
    /// 点を続ける．`None`は，線の切れ目である．
    fn push(&mut self, point: Option<Sample>) {
        match point {
            Some(point) => self.current.push(point),
            None => self.flush(),
        }
    }

    fn flush(&mut self) {
        let current = std::mem::take(&mut self.current);
        if current.len() >= 2 {
            self.finished.push(current);
        }
    }

    fn finish(mut self) -> Vec<Vec<Sample>> {
        self.flush();
        self.finished
    }
}

/// 区間の始まりの点は出力済みとして，区間の内側の点と，終わりの点を出力する．
fn segment<F: FnMut(f64) -> Option<Sample>>(
    eval: &mut F,
    [start, end]: [f64; 2],
    [first, last]: [Option<Sample>; 2],
    depth: u32,
    paths: &mut Paths,
) {
    if first.is_none() && last.is_none() {
        paths.push(None);
        return;
    }
    if depth >= MAX_DEPTH {
        paths.push(last);
        return;
    }
    let middle_t = f64::midpoint(start, end);
    let middle = eval(middle_t);
    if depth >= MIN_DEPTH && is_flat(eval, [start, end], [first, last], middle) {
        paths.push(last);
        return;
    }
    let next = depth.saturating_add(1);
    segment(eval, [start, middle_t], [first, middle], next, paths);
    segment(eval, [middle_t, end], [middle, last], next, paths);
}

/// 区間の端，中点，4分の1の点，4分の3の点が，弦から許容の距離に収まっているか．
fn is_flat<F: FnMut(f64) -> Option<Sample>>(
    eval: &mut F,
    [start, end]: [f64; 2],
    [first, last]: [Option<Sample>; 2],
    middle: Option<Sample>,
) -> bool {
    let (Some(first), Some(last), Some(middle)) = (first, last, middle) else {
        return false;
    };
    let middle_t = f64::midpoint(start, end);
    let quarter = eval(f64::midpoint(start, middle_t));
    let three_quarters = eval(f64::midpoint(middle_t, end));
    [quarter, Some(middle), three_quarters]
        .into_iter()
        .all(|sample| {
            sample.is_some_and(|(_, point)| distance_to_chord(point, first.1, last.1) <= TOLERANCE)
        })
}

/// 点から，弦(2点を結ぶ線分を延ばした直線)までの距離．弦が点に縮んでいるときは，その点までの距離．
fn distance_to_chord(point: Point, first: Point, last: Point) -> f64 {
    let chord = [last[0] - first[0], last[1] - first[1]];
    let offset = [point[0] - first[0], point[1] - first[1]];
    let length = chord[0].hypot(chord[1]);
    if length < f64::EPSILON {
        offset[0].hypot(offset[1])
    } else {
        (chord[0] * offset[1] - chord[1] * offset[0]).abs() / length
    }
}
