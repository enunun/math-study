import { isValidIdentifier, numberStatements } from './collect';
import type { Statement, StatementInfo } from './collect';
import { DocumentError } from './tree';
import type { JsxElement, Point } from './tree';

/** 式番号を付ける要素の種類の名前．定義や定理と同じ一覧で扱うため，`component`に入れる． */
const EQUATION_COMPONENT = 'Equation';

const LABEL_PATTERN = /\\label\{(?<id>[^{}]*)\}/gu;
/** MathJaxは，未定義の参照を「(???)」と描画する．本文の参照(`<Ref />`)を使わせるため，式の中では使わせない． */
const REFERENCE_PATTERN = /\\(?:eq)?ref(?![A-Za-z])/u;

/** 文書の中の1つの数式． */
interface EquationSource {
  tex: string;
  /** `$$…$$`の式．行内の`$…$`はfalse． */
  display: boolean;
  /** 段落から独立した`$$…$$`の式．式番号を付けられるのは，これだけである． */
  block: boolean;
  place: Point | undefined;
}

/** `\label`のある式． */
interface LabelledEquation<T extends EquationSource> {
  site: T;
  id: string;
  /** `\label`を取り除いたTeX． */
  tex: string;
}

/** 番号を付けた式． */
interface Equation<T extends EquationSource> {
  site: T;
  info: StatementInfo;
  /** `\label`を取り除き，`\tag`を加えたTeX． */
  tex: string;
}

/** `\label`の数と，式の種類が，正しいことを確かめる． */
function assertLabelPlacement(source: EquationSource, count: number): void {
  if (!source.block) {
    throw new DocumentError(
      String.raw`\labelは，段落から独立した別行立ての式($$…$$)だけに書ける．`,
      source.place,
    );
  }
  if (count > 1) {
    throw new DocumentError(
      String.raw`1つの式に書ける\labelは，1つだけである．複数行の式は，alignedで1つの式にする．`,
      source.place,
    );
  }
}

/** 式の中の`\label`から，識別子を読む．なければundefined． */
function readLabel(source: EquationSource): string | undefined {
  const ids = [...source.tex.matchAll(LABEL_PATTERN)].map((match) => match.groups?.id ?? '');
  const [id] = ids;
  if (id === undefined) {
    return undefined;
  }
  assertLabelPlacement(source, ids.length);
  if (!isValidIdentifier(id)) {
    throw new DocumentError(
      `式の識別子「${id}」は使えない．英数字で始め，英数字，ハイフン，アンダースコアだけを使う．`,
      source.place,
    );
  }
  return id;
}

/** `\label`のある式を，文書の順に集める．式の中の`\ref`と`\eqref`は，誤りにする． */
function findLabelledEquations<T extends EquationSource>(
  sites: readonly T[],
): LabelledEquation<T>[] {
  return sites.flatMap((site) => {
    if (REFERENCE_PATTERN.test(site.tex)) {
      throw new DocumentError(
        String.raw`式の中の\refと\eqrefは使えない．本文で<Ref to="識別子" />を使う．`,
        site.place,
      );
    }
    const id = readLabel(site);
    return id === undefined
      ? []
      : [{ site, id, tex: site.tex.replaceAll(LABEL_PATTERN, '').trim() }];
  });
}

/** 式に付ける番号の文字．テキストモードでは，`_`を`\_`と書く． */
function tagText(pageId: string, number: number): string {
  return `${pageId.replaceAll('_', String.raw`\_`)}-${number}`;
}

/**
 * `\label`のある式に，ページの識別子と，文書の順の連番から，番号を付ける．
 * 番号は，MathJaxの`\tag`で式の右に表示する．参照の文字は，「式(abs-3)」の形である．
 * 識別子が，定義や定理の識別子(`taken`)や，ほかの式の識別子と重なるときは，誤りにする．
 */
function numberEquations<T extends EquationSource>(
  labelled: readonly LabelledEquation<T>[],
  pageId: string,
  taken: ReadonlySet<string>,
): Equation<T>[] {
  const ids = new Set(taken);
  return labelled.map(({ site, id, tex }, index) => {
    if (ids.has(id)) {
      throw new DocumentError(
        `識別子「${id}」が，ページの中で重なっている．識別子は，ページの中で一意にする．`,
        site.place,
      );
    }
    ids.add(id);
    const number = index + 1;
    return {
      site,
      tex: `${tex}\n\\tag{${tagText(pageId, number)}}`,
      info: {
        component: EQUATION_COMPONENT,
        number,
        id,
        name: undefined,
        label: `式(${pageId}-${number})`,
        anchor: id,
      },
    };
  });
}

/** 定義や定理と式に，番号を付ける．識別子は，定義や定理と式で，ページの中で共通に一意である． */
function numberPageItems<T extends EquationSource>(
  nodes: readonly JsxElement[],
  labelled: readonly LabelledEquation<T>[],
  pageId: string,
): { statements: Statement[]; equations: Equation<T>[] } {
  const statements = numberStatements(nodes, pageId);
  const taken = new Set(statements.flatMap(({ id }) => (id === undefined ? [] : [id])));
  return { statements, equations: numberEquations(labelled, pageId, taken) };
}

export { EQUATION_COMPONENT, findLabelledEquations, numberPageItems };
export type { Equation, EquationSource, LabelledEquation };
