import {
  RHOMBOHEDRAL_ANGLE_MAX,
  RHOMBOHEDRAL_ANGLE_MIN,
  angle,
  cosOf,
  generalBasis,
  length,
  orthogonalBasis,
  planeCell,
  sinOf,
  spaceCell,
} from './bravais-cells';
import type { Basis, Centering, LatticeConstant } from './bravais-cells';
import type { ObjectTemplate } from './template-types';

/**
 * Bravais格子の単位胞．格子ベクトルの先端を点A，B，C(平面ではA，B)とし，座標を格子定数の媒介変数の式で
 * 書く．胞のほかの頂点と，体心・面心・底心の点は，点の式(`A + B`など)で書くので，格子定数を1つ変えれば
 * 胞全体が変わる．格子定数は範囲を持つので，図の作成のスライダーで動かせる．角度は度で書く．
 */

/** 既定の格子定数．3つの長さを違えて，どの辺がどの格子定数かを見分けやすくする． */
const A_LENGTH = 2;
const B_LENGTH = 2.5;
const C_LENGTH = 3;
const CUBIC_LENGTH = 2.5;

interface SpaceLattice {
  id: string;
  label: string;
  constants: readonly LatticeConstant[];
  basis: Basis;
  centering: Centering;
}

const TRICLINIC_ALPHA = 75;
const TRICLINIC_BETA = 80;
const TRICLINIC_GAMMA = 70;
const MONOCLINIC_BETA = 110;
const RHOMBOHEDRAL_ALPHA = 70;

const TRICLINIC: readonly LatticeConstant[] = [
  length('a', A_LENGTH),
  length('b', B_LENGTH),
  length('c', C_LENGTH),
  angle('angle_alpha', TRICLINIC_ALPHA),
  angle('angle_beta', TRICLINIC_BETA),
  angle('angle_gamma', TRICLINIC_GAMMA),
];
const MONOCLINIC: readonly LatticeConstant[] = [
  length('a', A_LENGTH),
  length('b', B_LENGTH),
  length('c', C_LENGTH),
  angle('angle_beta', MONOCLINIC_BETA),
];
/** 単斜晶は，b軸を2回軸にとる．aとcがなす角βだけが90度でない． */
const MONOCLINIC_BASIS: Basis = [
  ['a', '0', '0'],
  ['0', 'b', '0'],
  [`c*${cosOf('angle_beta')}`, '0', `c*${sinOf('angle_beta')}`],
];
const ORTHORHOMBIC: readonly LatticeConstant[] = [
  length('a', A_LENGTH),
  length('b', B_LENGTH),
  length('c', C_LENGTH),
];
const TETRAGONAL: readonly LatticeConstant[] = [length('a', A_LENGTH), length('c', C_LENGTH)];
/** 菱面体晶(三方晶のR格子)．3辺の長さaと，どの2辺もなす角αが等しい． */
const RHOMBOHEDRAL: readonly LatticeConstant[] = [
  length('a', CUBIC_LENGTH),
  angle('angle_alpha', RHOMBOHEDRAL_ALPHA, [RHOMBOHEDRAL_ANGLE_MIN, RHOMBOHEDRAL_ANGLE_MAX]),
];
const HEXAGONAL: readonly LatticeConstant[] = [length('a', A_LENGTH), length('c', C_LENGTH)];
/** 六方晶の単位胞は，aとbが120度をなす菱形の柱である．六角柱は，この胞3つでできる． */
const HEXAGONAL_BASIS: Basis = [
  ['a', '0', '0'],
  ['-a/2', 'a*sqrt(3)/2', '0'],
  ['0', '0', 'c'],
];
const CUBIC: readonly LatticeConstant[] = [length('a', CUBIC_LENGTH)];

const TRICLINIC_BASIS = generalBasis(['a', 'b', 'c'], ['angle_alpha', 'angle_beta', 'angle_gamma']);
const ORTHORHOMBIC_BASIS = orthogonalBasis('a', 'b', 'c');
const TETRAGONAL_BASIS = orthogonalBasis('a', 'a', 'c');
const RHOMBOHEDRAL_BASIS = generalBasis(
  ['a', 'a', 'a'],
  ['angle_alpha', 'angle_alpha', 'angle_alpha'],
);
const CUBIC_BASIS = orthogonalBasis('a', 'a', 'a');

/** 空間の14種類のBravais格子．晶系の順(三斜，単斜，直方，正方，三方，六方，立方)に並べる． */
const SPACE_LATTICES: readonly SpaceLattice[] = [
  { id: 'aP', label: '三斜(aP)', constants: TRICLINIC, basis: TRICLINIC_BASIS, centering: 'P' },
  {
    id: 'mP',
    label: '単斜・単純(mP)',
    constants: MONOCLINIC,
    basis: MONOCLINIC_BASIS,
    centering: 'P',
  },
  {
    id: 'mS',
    label: '単斜・底心(mS)',
    constants: MONOCLINIC,
    basis: MONOCLINIC_BASIS,
    centering: 'S',
  },
  {
    id: 'oP',
    label: '直方・単純(oP)',
    constants: ORTHORHOMBIC,
    basis: ORTHORHOMBIC_BASIS,
    centering: 'P',
  },
  {
    id: 'oS',
    label: '直方・底心(oS)',
    constants: ORTHORHOMBIC,
    basis: ORTHORHOMBIC_BASIS,
    centering: 'S',
  },
  {
    id: 'oI',
    label: '直方・体心(oI)',
    constants: ORTHORHOMBIC,
    basis: ORTHORHOMBIC_BASIS,
    centering: 'I',
  },
  {
    id: 'oF',
    label: '直方・面心(oF)',
    constants: ORTHORHOMBIC,
    basis: ORTHORHOMBIC_BASIS,
    centering: 'F',
  },
  {
    id: 'tP',
    label: '正方・単純(tP)',
    constants: TETRAGONAL,
    basis: TETRAGONAL_BASIS,
    centering: 'P',
  },
  {
    id: 'tI',
    label: '正方・体心(tI)',
    constants: TETRAGONAL,
    basis: TETRAGONAL_BASIS,
    centering: 'I',
  },
  {
    id: 'hR',
    label: '三方・菱面体(hR)',
    constants: RHOMBOHEDRAL,
    basis: RHOMBOHEDRAL_BASIS,
    centering: 'P',
  },
  { id: 'hP', label: '六方(hP)', constants: HEXAGONAL, basis: HEXAGONAL_BASIS, centering: 'P' },
  { id: 'cP', label: '立方・単純(cP)', constants: CUBIC, basis: CUBIC_BASIS, centering: 'P' },
  { id: 'cI', label: '立方・体心(cI)', constants: CUBIC, basis: CUBIC_BASIS, centering: 'I' },
  { id: 'cF', label: '立方・面心(cF)', constants: CUBIC, basis: CUBIC_BASIS, centering: 'F' },
];

interface PlaneLattice {
  id: string;
  label: string;
  constants: readonly LatticeConstant[];
  basis: Basis;
  centered: boolean;
}

const OBLIQUE_GAMMA = 70;

/** 平面の5種類のBravais格子． */
const PLANE_LATTICES: readonly PlaneLattice[] = [
  {
    id: 'mp',
    label: '斜方(mp)',
    constants: [length('a', A_LENGTH), length('b', B_LENGTH), angle('angle_gamma', OBLIQUE_GAMMA)],
    basis: [
      ['a', '0'],
      [`b*${cosOf('angle_gamma')}`, `b*${sinOf('angle_gamma')}`],
    ],
    centered: false,
  },
  {
    id: 'op',
    label: '長方形(op)',
    constants: [length('a', A_LENGTH), length('b', B_LENGTH)],
    basis: [
      ['a', '0'],
      ['0', 'b'],
    ],
    centered: false,
  },
  {
    id: 'oc',
    label: '面心長方形(oc)',
    constants: [length('a', A_LENGTH), length('b', C_LENGTH)],
    basis: [
      ['a', '0'],
      ['0', 'b'],
    ],
    centered: true,
  },
  {
    id: 'tp',
    label: '正方形(tp)',
    constants: [length('a', A_LENGTH)],
    basis: [
      ['a', '0'],
      ['0', 'a'],
    ],
    centered: false,
  },
  {
    id: 'hp',
    label: '六角(hp)',
    constants: [length('a', A_LENGTH)],
    basis: [
      ['a', '0'],
      ['-a/2', 'a*sqrt(3)/2'],
    ],
    centered: false,
  },
];

/** Bravais格子のテンプレート．平面の5種類，空間の14種類である． */
const BRAVAIS_TEMPLATES: readonly ObjectTemplate[] = [
  ...PLANE_LATTICES.map(({ id, label, constants, basis, centered }) => ({
    id: `bravais-${id}`,
    label,
    kind: 'plane' as const,
    objects: planeCell(constants, basis, centered),
  })),
  ...SPACE_LATTICES.map(({ id, label, constants, basis, centering }) => ({
    id: `bravais-${id}`,
    label,
    kind: 'space' as const,
    objects: spaceCell(constants, basis, centering),
  })),
];

export { BRAVAIS_TEMPLATES };
